//! Methodology readiness: how far the brain follows each method it uses.
//!
//! One section per method or channel (PARA, Zettelkasten, Obsidian, RSS,
//! arXiv), each a list of checks run on the live brain. A check is `ready`,
//! `partial` (it works, but there is a backlog) or `missing` (not set up yet),
//! and says what to run to fix it. The rules are the ones in the `zk-method`
//! skill and in `docs/methodology.md`.
#![allow(clippy::missing_errors_doc)]
use std::collections::{HashMap, HashSet};

use loco_rs::model::ModelResult;
use sea_orm::{ConnectionTrait, EntityTrait};
use serde::Serialize;

use crate::models::{
    areas, feeds, hippos, note_links, notes, plasts, project_notes, projects, resources, tasks,
};

/// A fleeting note older than this is backlog, not inbox.
const INBOX_DAYS: i64 = 14;
/// A feed not pulled for this long is stale.
const FEED_DAYS: i64 = 7;
/// Past this many words a permanent note is likely more than one idea.
const ATOMIC_WORDS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ready,
    Partial,
    Missing,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
    /// What to run, when the check is not ready.
    pub fix: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Section {
    pub name: &'static str,
    /// The worst status of its checks.
    pub status: Status,
    pub checks: Vec<Check>,
}

#[derive(Debug, Serialize)]
pub struct Readiness {
    pub ready: usize,
    pub total: usize,
    pub sections: Vec<Section>,
}

fn check(name: &'static str, ok: bool, partial: bool, detail: String, fix: &str) -> Check {
    let status = if ok {
        Status::Ready
    } else if partial {
        Status::Partial
    } else {
        Status::Missing
    };
    Check {
        name,
        status,
        detail,
        fix: (status != Status::Ready).then(|| fix.to_string()),
    }
}

fn section(name: &'static str, checks: Vec<Check>) -> Section {
    Section {
        name,
        status: checks
            .iter()
            .map(|c| c.status)
            .max()
            .unwrap_or(Status::Ready),
        checks,
    }
}

/// `n` things, with a few of them named.
fn some(n: usize, what: &str, names: &[String]) -> String {
    let shown: Vec<String> = names
        .iter()
        .take(3)
        .map(|n| {
            if n.chars().count() > 40 {
                format!("{}…", n.chars().take(39).collect::<String>())
            } else {
                n.clone()
            }
        })
        .collect();
    let more = if names.len() > 3 { ", ..." } else { "" };
    if shown.is_empty() {
        format!("{n} {what}")
    } else {
        format!("{n} {what}: {}{more}", shown.join(", "))
    }
}

/// Ready when `items` is empty, else partial, naming a few of them.
fn listed(name: &'static str, items: &[String], ok: &str, what: &str, fix: &str) -> Check {
    if items.is_empty() {
        check(name, true, true, ok.to_string(), fix)
    } else {
        check(name, false, true, some(items.len(), what, items), fix)
    }
}

/// Everything the checks look at, loaded once.
struct Brain {
    areas: Vec<areas::Model>,
    projects: Vec<projects::Model>,
    tasks: Vec<tasks::Model>,
    resources: Vec<resources::Model>,
    notes: Vec<notes::Model>,
    links: Vec<note_links::Model>,
    pulled: Vec<project_notes::Model>,
    feeds: Vec<feeds::Model>,
    today: chrono::NaiveDate,
}

impl Brain {
    async fn load<C: ConnectionTrait>(db: &C) -> ModelResult<Self> {
        Ok(Self {
            areas: areas::Entity::find().all(db).await?,
            projects: projects::Entity::find().all(db).await?,
            tasks: tasks::Entity::find().all(db).await?,
            resources: resources::Entity::find().all(db).await?,
            notes: notes::Entity::find().all(db).await?,
            links: note_links::Entity::find().all(db).await?,
            pulled: project_notes::Entity::find().all(db).await?,
            feeds: feeds::Entity::find().all(db).await?,
            today: chrono::Utc::now().date_naive(),
        })
    }

    fn active_projects(&self) -> impl Iterator<Item = &projects::Model> {
        self.projects.iter().filter(|p| p.archived_at.is_none())
    }

    fn of_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a notes::Model> {
        self.notes.iter().filter(move |n| n.kind == kind)
    }

    fn para(&self) -> Section {
        let areas = self
            .areas
            .iter()
            .filter(|a| a.archived_at.is_none())
            .count();
        let active: Vec<&projects::Model> = self
            .active_projects()
            .filter(|p| p.status.as_deref() != Some("done"))
            .collect();
        let undated: Vec<String> = active
            .iter()
            .filter(|p| p.due_on.is_none())
            .map(|p| format!("#{} {}", p.id, p.name))
            .collect();
        let open: HashSet<i64> = self
            .tasks
            .iter()
            .filter(|t| !t.done)
            .map(|t| t.project_id)
            .collect();
        let stuck: Vec<String> = active
            .iter()
            .filter(|p| p.status.as_deref() == Some("active") && !open.contains(&p.id))
            .map(|p| format!("#{} {}", p.id, p.name))
            .collect();
        let done: Vec<String> = self
            .active_projects()
            .filter(|p| p.status.as_deref() == Some("done"))
            .map(|p| format!("#{} {}", p.id, p.name))
            .collect();
        let overdue: Vec<String> = self
            .tasks
            .iter()
            .filter(|t| !t.done && t.due_on.is_some_and(|d| d < self.today))
            .map(|t| format!("#{} {}", t.id, t.title))
            .collect();
        let unfiled: Vec<String> = self
            .resources
            .iter()
            .filter(|r| r.archived_at.is_none() && r.area_id.is_none() && r.project_id.is_none())
            .map(|r| format!("#{} {}", r.id, r.title))
            .collect();
        section(
            "PARA",
            vec![
                check(
                    "areas",
                    areas > 0,
                    false,
                    format!("{areas} active area(s)"),
                    "ztk area -c -n NAME",
                ),
                listed(
                    "projects have a deadline",
                    &undated,
                    &format!("{} open project(s), all dated", active.len()),
                    "without a due date",
                    "ztk project -u ID --due YYYY-MM-DD (no end date: it is an area)",
                ),
                listed(
                    "active projects have a next action",
                    &stuck,
                    "every active project has an open task",
                    "with no open task",
                    "ztk task -c -t TITLE -p ID, or ztk project -u ID --status waiting",
                ),
                listed(
                    "done projects archived",
                    &done,
                    "no done project left out",
                    "done but not archived",
                    "ztk project --archive ID",
                ),
                listed(
                    "no overdue tasks",
                    &overdue,
                    "nothing overdue",
                    "overdue",
                    "ztk task --done ID, or ztk task -u ID --due YYYY-MM-DD",
                ),
                listed(
                    "resources filed",
                    &unfiled,
                    &format!("{} resource(s), all filed", self.resources.len()),
                    "in no area or project",
                    "ztk resource -u ID -a AREA_ID",
                ),
            ],
        )
    }

    fn zettelkasten(&self) -> Section {
        let permanent: Vec<&notes::Model> = self.of_kind("permanent").collect();
        let linked: HashSet<i64> = self
            .links
            .iter()
            .flat_map(|l| [l.source_id, l.target_id])
            .collect();
        let cutoff = chrono::Utc::now() - chrono::Duration::days(INBOX_DAYS);
        let stale: Vec<String> = self
            .of_kind("fleeting")
            .filter(|n| n.created_at < cutoff)
            .map(|n| format!("{} {}", n.zid, n.title))
            .collect();
        let orphans: Vec<String> = permanent
            .iter()
            .filter(|n| !linked.contains(&n.id))
            .map(|n| format!("{} {}", n.zid, n.title))
            .collect();
        let long: Vec<String> = permanent
            .iter()
            .filter(|n| {
                n.body.split_whitespace().count() > ATOMIC_WORDS
                    || n.body.matches("\n## ").count() + usize::from(n.body.starts_with("## ")) > 1
            })
            .map(|n| format!("{} {}", n.zid, n.title))
            .collect();
        let loose_literature: Vec<String> = self
            .of_kind("literature")
            .filter(|n| n.resource_id.is_none())
            .map(|n| format!("{} {}", n.zid, n.title))
            .collect();
        let hubs = self.of_kind("hub").count();
        let pulled: HashSet<i64> = self.pulled.iter().map(|p| p.project_id).collect();
        let unlinked_projects: Vec<String> = self
            .active_projects()
            .filter(|p| !pulled.contains(&p.id))
            .map(|p| format!("#{} {}", p.id, p.name))
            .collect();
        section(
            "Zettelkasten",
            vec![
                check(
                    "slipbox started",
                    !permanent.is_empty(),
                    false,
                    format!("{} permanent note(s)", permanent.len()),
                    "ztk note -c -t \"A claim in your words\" -k permanent -e",
                ),
                listed(
                    "inbox drains",
                    &stale,
                    &format!("no fleeting note older than {INBOX_DAYS} days"),
                    &format!("fleeting notes older than {INBOX_DAYS} days"),
                    "ztk box inbox: turn each into a literature/permanent note, or delete it",
                ),
                listed(
                    "no orphan permanent notes",
                    &orphans,
                    "every permanent note is linked",
                    "linked to nothing",
                    "ztk link ZID OTHER -w \"why they belong together\"",
                ),
                listed(
                    "permanent notes are atomic",
                    &long,
                    &format!("none over {ATOMIC_WORDS} words or with several sections"),
                    "likely hold more than one idea",
                    "split each into one note per idea, continuing it (ztk note -c -a ZID)",
                ),
                check(
                    "entry points exist",
                    hubs > 0 || permanent.len() <= 5,
                    true,
                    format!("{hubs} hub note(s) for {} permanent", permanent.len()),
                    "ztk note -c -t TOPIC -k hub, listing its notes with a line each",
                ),
                listed(
                    "literature notes hang off a resource",
                    &loose_literature,
                    "every literature note points at its source's resource",
                    "cite a source with no resource",
                    "ztk note -u ZID -r RESOURCE_ID",
                ),
                listed(
                    "projects pull notes in",
                    &unlinked_projects,
                    "every active project links the notes it uses",
                    "link no notes (their knowledge goes when they are archived)",
                    "ztk project --pull ID --note ZID -w why",
                ),
            ],
        )
    }

    fn obsidian(&self) -> Section {
        let zids: HashSet<&str> = self.notes.iter().map(|n| n.zid.as_str()).collect();
        let titles: HashSet<String> = self.notes.iter().map(|n| n.title.to_lowercase()).collect();
        let dangling: Vec<String> = self
            .notes
            .iter()
            .filter(|n| {
                notes::wikilinks(&n.body)
                    .iter()
                    .any(|k| !zids.contains(k.as_str()) && !titles.contains(&k.to_lowercase()))
            })
            .map(|n| format!("{} {}", n.zid, n.title))
            .collect();
        section(
            "Obsidian in/out",
            vec![
                Check {
                    name: "vault round trip",
                    status: Status::Ready,
                    detail: "ztk obsidian export DIR / ztk obsidian import DIR [--dry-run]".into(),
                    fix: None,
                },
                listed(
                    "wikilinks resolve",
                    &dangling,
                    "every [[link]] has a note behind it",
                    "link to notes that do not exist yet",
                    "write the missing notes (links are picked up when they arrive)",
                ),
            ],
        )
    }

    fn rss(&self) -> Section {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(FEED_DAYS);
        let stale: Vec<String> = self
            .feeds
            .iter()
            .filter(|f| f.last_fetched_at.is_none_or(|t| t < cutoff))
            .map(|f| format!("#{} {}", f.id, f.title.as_deref().unwrap_or(&f.url)))
            .collect();
        let unfiled = self
            .feeds
            .iter()
            .filter(|f| f.area_id.is_none() && f.project_id.is_none())
            .count();
        let publishable = self.of_kind("permanent").count();
        section(
            "RSS in/out",
            vec![
                check(
                    "feeds subscribed",
                    !self.feeds.is_empty(),
                    false,
                    format!("{} feed(s), {unfiled} filed nowhere", self.feeds.len()),
                    "ztk rss add URL -a AREA_ID",
                ),
                check(
                    "feeds pulled",
                    !self.feeds.is_empty() && stale.is_empty(),
                    !self.feeds.is_empty(),
                    if self.feeds.is_empty() {
                        "no feeds".into()
                    } else if stale.is_empty() {
                        format!("all pulled in the last {FEED_DAYS} days")
                    } else {
                        some(
                            stale.len(),
                            &format!("not pulled in {FEED_DAYS} days"),
                            &stale,
                        )
                    },
                    "ztk rss pull (daily, e.g. from cron)",
                ),
                check(
                    "slipbox feed out",
                    publishable > 0,
                    true,
                    format!("{publishable} permanent note(s) in GET /api/slipbox/feed.xml"),
                    "write permanent notes; ztk rss out -o feed.xml",
                ),
            ],
        )
    }

    fn arxiv(&self) -> Section {
        let papers: Vec<&resources::Model> = self
            .resources
            .iter()
            .filter(|r| {
                r.external_id
                    .as_deref()
                    .is_some_and(|e| e.starts_with("arxiv:"))
            })
            .collect();
        let unfiled: Vec<String> = papers
            .iter()
            .filter(|r| r.archived_at.is_none() && r.area_id.is_none() && r.project_id.is_none())
            .map(|r| {
                r.citekey
                    .clone()
                    .map_or_else(|| r.title.clone(), |k| format!("@{k}"))
            })
            .collect();
        let mut notes_per: HashMap<i64, usize> = HashMap::new();
        for n in self.of_kind("literature") {
            if let Some(id) = n.resource_id {
                *notes_per.entry(id).or_default() += 1;
            }
        }
        let unread: Vec<String> = papers
            .iter()
            .filter(|r| r.archived_at.is_none() && !notes_per.contains_key(&r.id))
            .map(|r| {
                r.citekey
                    .clone()
                    .map_or_else(|| r.title.clone(), |k| format!("@{k}"))
            })
            .collect();
        section(
            "arXiv in",
            vec![
                check(
                    "papers ingested",
                    !papers.is_empty(),
                    false,
                    format!("{} paper(s) from arXiv", papers.len()),
                    "ztk arxiv add ID... [-a AREA_ID] [--note]",
                ),
                check(
                    "papers filed",
                    !papers.is_empty() && unfiled.is_empty(),
                    !papers.is_empty(),
                    if papers.is_empty() {
                        "no papers".into()
                    } else if unfiled.is_empty() {
                        "every paper is in an area or project".into()
                    } else {
                        some(unfiled.len(), "in no area or project", &unfiled)
                    },
                    "ztk resource -u ID -a AREA_ID",
                ),
                check(
                    "papers processed",
                    !papers.is_empty() && unread.is_empty(),
                    !papers.is_empty(),
                    if papers.is_empty() {
                        "no papers".into()
                    } else if unread.is_empty() {
                        "every paper has a literature note".into()
                    } else {
                        some(unread.len(), "without a literature note", &unread)
                    },
                    "ztk note -c -t \"What it says, in your words\" -k literature -r RESOURCE_ID",
                ),
            ],
        )
    }
}

/// Each Hippo's rhizome is whole, and each of its fixations has notes.
/// Hippo identities are optional: with none, the section is ready.
async fn hippo<C: ConnectionTrait>(db: &C) -> ModelResult<Section> {
    let all = hippos::Entity::listing().all(db).await?;
    if all.is_empty() {
        return Ok(section(
            "Hippo rhizome",
            vec![Check {
                name: "Hippo identity",
                status: Status::Ready,
                detail: "none (optional: ztk hippo -c -n NAME --trait adhd --fixation TOPIC)"
                    .into(),
                fix: None,
            }],
        ));
    }
    let mut broken = Vec::new();
    let mut empty = Vec::new();
    for h in &all {
        let map = h.map(db).await?;
        if map.components.len() > 1 {
            broken.push(format!(
                "#{} {} ({} pieces)",
                h.id,
                h.name,
                map.components.len()
            ));
        }
        for p in map.plateaus.iter().filter(|p| p.notes.is_empty()) {
            empty.push(format!("{}: {}", h.name, p.fixation));
        }
    }
    Ok(section(
        "Hippo rhizome",
        vec![
            listed(
                "rhizome is whole",
                &broken,
                &format!("{} Hippo(s), every rhizome in one piece", all.len()),
                "in pieces",
                "ztk hippo --weave ID",
            ),
            listed(
                "fixations have notes",
                &empty,
                "every fixation has notes around it",
                "fixations with no note yet",
                "ztk note -c \"a thought\" -H ID --tag FIXATION",
            ),
        ],
    ))
}

/// Plasticity is practice, so the one check is whether it happens.
async fn plasticity<C: ConnectionTrait>(db: &C) -> ModelResult<Section> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
    let all = plasts::Entity::listing(None, db).await?;
    let fresh = all
        .iter()
        .filter(|p| p.practiced_at.is_some_and(|t| t >= cutoff))
        .count();
    Ok(section(
        "Plasticity",
        vec![check(
            "practised this week",
            fresh > 0,
            true,
            format!(
                "{fresh} of {} plasts practised in the last 7 days",
                all.len()
            ),
            "ztk plast go (five quick exercises)",
        )],
    ))
}

/// Run every check on the brain.
pub async fn report<C: ConnectionTrait>(db: &C) -> ModelResult<Readiness> {
    let brain = Brain::load(db).await?;
    let sections = vec![
        brain.para(),
        brain.zettelkasten(),
        brain.obsidian(),
        brain.rss(),
        brain.arxiv(),
        hippo(db).await?,
        plasticity(db).await?,
    ];
    let checks = sections.iter().flat_map(|s| &s.checks);
    Ok(Readiness {
        ready: checks.clone().filter(|c| c.status == Status::Ready).count(),
        total: checks.count(),
        sections,
    })
}
