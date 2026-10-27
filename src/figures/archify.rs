//! The PARA tree as [archify](https://github.com/tt-a1i/archify) JSON IR.
//!
//! Emits a `diagram_type: "tree"` document (schema version 1): one root for
//! the brain, areas below it, then projects, their tasks, and resources under
//! the project or area they belong to. Unfiled resources hang from their own
//! node. Render it with:
//!
//! ```sh
//! node <archify>/bin/archify.mjs finalize tree para.tree.json para.html --quality showcase --json
//! ```

use serde::Serialize;

use super::{area_id, project_id, resource_id, task_id, Brain};

const ROOT: &str = "brain";
const UNFILED: &str = "unfiled";
/// Long sublabels crowd archify's layout and trip its label checks.
const SUBLABEL_MAX: usize = 60;

#[derive(Debug, Clone, Serialize)]
pub struct Options {
    /// `meta.title`.
    pub title: String,
    /// `meta.output`: where `finalize` writes the HTML, relative to the working directory.
    pub output: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            title: "Second brain".to_string(),
            output: "para.html".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Tree {
    pub schema_version: u8,
    pub diagram_type: &'static str,
    pub meta: Meta,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Serialize)]
pub struct Meta {
    pub title: String,
    pub subtitle: String,
    pub output: String,
    pub quality_profile: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
}

impl Node {
    fn new(id: String, label: &str, parent: Option<String>, sublabel: Option<String>) -> Self {
        Self {
            id,
            label: label.to_string(),
            parent,
            sublabel: sublabel.map(|s| clip(&s)).filter(|s| !s.is_empty()),
        }
    }
}

/// Build the tree IR for a snapshot.
#[must_use]
pub fn tree(brain: &Brain, opts: &Options) -> Tree {
    let mut nodes = vec![Node::new(
        ROOT.to_string(),
        &opts.title,
        None,
        Some(summary(brain)),
    )];

    for a in &brain.areas {
        let sub = join([
            a.archived_at.map(|_| "archived".to_string()),
            a.description.clone(),
        ]);
        nodes.push(Node::new(area_id(a.id), &a.name, Some(ROOT.into()), sub));
    }
    for p in brain.placed_projects() {
        let sub = join([
            p.archived_at.map(|_| "archived".to_string()),
            p.status.clone(),
        ]);
        nodes.push(Node::new(
            project_id(p.id),
            &p.name,
            Some(area_id(p.area_id)),
            sub,
        ));
        for t in brain.tasks.iter().filter(|t| t.project_id == p.id) {
            let sub = join([
                t.done.then(|| "done".to_string()),
                t.due_on.map(|d| format!("due {d}")),
            ]);
            nodes.push(Node::new(
                task_id(t.id),
                &t.title,
                Some(project_id(p.id)),
                sub,
            ));
        }
    }

    let mut unfiled = Vec::new();
    for r in &brain.resources {
        let sub = join([r.archived_at.map(|_| "archived".to_string()), r.url.clone()]);
        match brain.resource_home(r) {
            Some(home) => nodes.push(Node::new(resource_id(r.id), &r.title, Some(home.id()), sub)),
            None => unfiled.push(Node::new(
                resource_id(r.id),
                &r.title,
                Some(UNFILED.into()),
                sub,
            )),
        }
    }
    if !unfiled.is_empty() {
        nodes.push(Node::new(
            UNFILED.to_string(),
            "Unfiled resources",
            Some(ROOT.into()),
            None,
        ));
        nodes.extend(unfiled);
    }

    Tree {
        schema_version: 1,
        diagram_type: "tree",
        meta: Meta {
            title: opts.title.clone(),
            subtitle: "PARA exported by ztk. Links show containment: area, project, task, \
                       resource."
                .to_string(),
            output: opts.output.clone(),
            quality_profile: "showcase",
        },
        nodes,
    }
}

fn summary(brain: &Brain) -> String {
    format!(
        "{} areas, {} projects, {} tasks, {} resources",
        brain.areas.len(),
        brain.placed_projects().count(),
        brain
            .tasks
            .iter()
            .filter(|t| brain.has_project(t.project_id))
            .count(),
        brain.resources.len()
    )
}

fn join<const N: usize>(parts: [Option<String>; N]) -> Option<String> {
    let parts: Vec<String> = parts
        .into_iter()
        .flatten()
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

fn clip(s: &str) -> String {
    if s.chars().count() <= SUBLABEL_MAX {
        return s.to_string();
    }
    let mut out: String = s.chars().take(SUBLABEL_MAX - 1).collect();
    out.push('…');
    out
}
