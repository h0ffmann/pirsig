//! The brain as a [graphify](https://github.com/safishamsi/graphify) knowledge graph.
//!
//! Emits graphify's `graph.json` (networkx node-link data with `links`): every
//! area, project, task and resource is a node, and containment is a
//! `contains` edge. Every edge comes from ztk's own records, so all of them
//! are `EXTRACTED`. Point graphify at it to cluster, query and draw the brain:
//!
//! ```sh
//! ztk graph -o graphify-out/graph.json
//! graphify cluster-only . --no-label    # communities, GRAPH_REPORT.md, graph.html
//! graphify query "what feeds the marathon?"
//! ```

use serde::Serialize;

use super::{area_id, project_id, resource_id, task_id, Brain};

#[derive(Debug, Serialize)]
pub struct Graph {
    pub directed: bool,
    pub multigraph: bool,
    pub graph: GraphMeta,
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize)]
pub struct GraphMeta {
    pub generator: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    /// graphify's node type; ztk records are `concept`s, resources `document`s.
    pub file_type: &'static str,
    /// Where the node came from: `ztk/<kind>s/<id>`, the record's REST path.
    pub source_file: String,
    /// `area`, `project`, `task` or `resource`.
    pub kind: &'static str,
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub source: String,
    pub target: String,
    pub relation: &'static str,
    pub confidence: &'static str,
    pub confidence_score: f64,
    pub weight: f64,
    pub source_file: String,
}

impl Node {
    fn new(kind: &'static str, id: i64, label: &str, archived: bool) -> Self {
        Self {
            id: format!("{kind}-{id}"),
            label: label.to_string(),
            file_type: if kind == "resource" {
                "document"
            } else {
                "concept"
            },
            source_file: format!("ztk/{kind}s/{id}"),
            kind,
            archived,
            status: None,
            done: None,
            due_on: None,
            url: None,
        }
    }
}

fn contains(source: String, target: String, source_file: &str) -> Link {
    Link {
        source,
        target,
        relation: "contains",
        confidence: "EXTRACTED",
        confidence_score: 1.0,
        weight: 1.0,
        source_file: source_file.to_string(),
    }
}

/// Build the graph for a snapshot.
#[must_use]
pub fn graph(brain: &Brain) -> Graph {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    for a in &brain.areas {
        nodes.push(Node::new("area", a.id, &a.name, a.archived_at.is_some()));
    }
    for p in brain.placed_projects() {
        let mut node = Node::new("project", p.id, &p.name, p.archived_at.is_some());
        node.status.clone_from(&p.status);
        links.push(contains(
            area_id(p.area_id),
            project_id(p.id),
            &node.source_file,
        ));
        nodes.push(node);
        for t in brain.tasks.iter().filter(|t| t.project_id == p.id) {
            let mut node = Node::new("task", t.id, &t.title, false);
            node.done = Some(t.done);
            node.due_on = t.due_on.map(|d| d.to_string());
            links.push(contains(project_id(p.id), task_id(t.id), &node.source_file));
            nodes.push(node);
        }
    }
    for r in &brain.resources {
        let mut node = Node::new("resource", r.id, &r.title, r.archived_at.is_some());
        node.url.clone_from(&r.url);
        if let Some(home) = brain.resource_home(r) {
            links.push(contains(home.id(), resource_id(r.id), &node.source_file));
        }
        nodes.push(node);
    }

    Graph {
        directed: true,
        multigraph: false,
        graph: GraphMeta { generator: "ztk" },
        nodes,
        links,
    }
}
