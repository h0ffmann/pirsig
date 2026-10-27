//! `ztk.pkm.obsidian`: the brain as an Obsidian vault, out and back in.
//!
//! The vault keeps PARA and the slipbox apart, as the method asks: PARA items
//! are folders (`Projects/`, `Areas/`, `Resources/`, `Archive/`), notes live
//! flat in `Zettelkasten/`, ordered by zid and connected by links. Every file
//! ztk writes carries a `ztk:` front matter key; export only ever replaces or
//! removes those files, never the person's own.
//!
//! Links are rewritten both ways so they resolve on each side: ztk's `[[1a]]`
//! or `[[Some title]]` becomes Obsidian's `[[1a Some title|1a]]` (a file stem),
//! and an Obsidian `[[file stem]]` comes back as `[[zid]]`.
#![allow(clippy::missing_errors_doc)]
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Write,
    path::{Path, PathBuf},
};

use loco_rs::{model::ModelResult, Error, Result};
use sea_orm::{
    prelude::{Date, DateTimeWithTimeZone},
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter,
    TransactionTrait,
};
use serde::Serialize;
use serde_yaml::Value;

use crate::{
    dtos::{
        areas::{CreateArea, UpdateArea},
        common::Patch,
        notes::{CreateNote, UpdateNote},
        projects::{CreateProject, UpdateProject},
        resources::{CreateResource, UpdateResource},
        tasks::{CreateTask, UpdateTask},
    },
    models::{
        areas, hippos, notes,
        notes::{rewrite_wikilinks_with, Linked},
        projects, resources, tasks,
    },
};

const NOTES: &str = "Zettelkasten";
const AREAS: &str = "Areas";
const PROJECTS: &str = "Projects";
const RESOURCES: &str = "Resources";
const ARCHIVE: &str = "Archive";

#[derive(Debug, Default, Serialize)]
pub struct ExportReport {
    pub notes: usize,
    pub areas: usize,
    pub projects: usize,
    pub resources: usize,
    /// ztk files from an earlier export whose item is gone or renamed.
    pub removed: usize,
}

#[derive(Debug, Default, Serialize)]
pub struct ImportReport {
    pub notes_created: usize,
    pub notes_updated: usize,
    pub areas: usize,
    pub projects: usize,
    pub tasks: usize,
    pub resources: usize,
    pub links: usize,
    /// Notes changed to fit the method, and lines that could not be read.
    pub warnings: Vec<String>,
    /// True when nothing was written (`--dry-run`).
    pub dry_run: bool,
}

// ---------------------------------------------------------------------------
// out

/// `21a Some title`: the file stem of a note, safe on every file system.
#[must_use]
pub fn note_stem(note: &notes::Model) -> String {
    format!("{} {}", note.zid, safe(&note.title))
}

/// A title with the characters that trouble file systems and Obsidian links dropped.
fn safe(title: &str) -> String {
    let s: String = title
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '#' | '^' | '[' | ']'
            )
        })
        .collect();
    let s: String = s
        .trim()
        .trim_start_matches('.')
        .trim()
        .chars()
        .take(120)
        .collect();
    if s.is_empty() {
        "untitled".to_string()
    } else {
        s
    }
}

fn yaml(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn link(path: &str, label: &str) -> String {
    format!("[[{path}|{label}]]")
}

fn front(kind: &str, id: i64, title: &str, archived: bool) -> String {
    let mut md = String::from("---\n");
    let _ = writeln!(md, "ztk: {kind}");
    let _ = writeln!(md, "ztk_id: {id}");
    let _ = writeln!(md, "title: {}", yaml(title));
    if archived {
        md.push_str("archived: true\n");
    }
    md
}

/// Close the front matter and open the body with the title and description.
fn heading(md: &mut String, title: &str, text: Option<&str>) {
    md.push_str("---\n\n");
    let _ = writeln!(md, "# {title}\n");
    if let Some(t) = text {
        let _ = writeln!(md, "{}\n", t.trim_end());
    }
}

fn section(md: &mut String, heading: &str, lines: impl Iterator<Item = String>) {
    let lines: Vec<String> = lines.collect();
    if lines.is_empty() {
        return;
    }
    let _ = writeln!(md, "## {heading}\n");
    for l in lines {
        let _ = writeln!(md, "- {l}");
    }
    md.push('\n');
}

/// Everything export needs, loaded once, with each PARA item's path in the
/// vault (without `.md`); two items with one name get their id appended.
struct Snapshot {
    areas: Vec<areas::Model>,
    projects: Vec<projects::Model>,
    resources: Vec<resources::Model>,
    tasks: Vec<tasks::Model>,
    notes: Vec<notes::Model>,
    hippos: HashMap<i64, String>,
    paths: HashMap<(&'static str, i64), String>,
}

impl Snapshot {
    async fn load(db: &DatabaseConnection, archived: bool) -> ModelResult<Self> {
        let keep = |a: &Option<DateTimeWithTimeZone>| archived || a.is_none();
        let mut snap = Self {
            areas: areas::Entity::find().all(db).await?,
            projects: projects::Entity::find().all(db).await?,
            resources: resources::Entity::find().all(db).await?,
            tasks: tasks::Entity::listing(None, None).all(db).await?,
            notes: notes::Entity::tree(db).await?,
            hippos: hippos::Entity::find()
                .all(db)
                .await?
                .into_iter()
                .map(|h| (h.id, h.name))
                .collect(),
            paths: HashMap::new(),
        };
        snap.areas.retain(|a| keep(&a.archived_at));
        snap.projects.retain(|p| keep(&p.archived_at));
        snap.resources.retain(|r| keep(&r.archived_at));
        let mut taken = HashSet::new();
        let mut place = |folder: &'static str, archived: bool, name: &str, id: i64| {
            let base = if archived {
                format!("{ARCHIVE}/{folder}")
            } else {
                folder.to_string()
            };
            let mut path = format!("{base}/{}", safe(name));
            if !taken.insert(path.to_lowercase()) {
                path = format!("{base}/{} ({id})", safe(name));
                taken.insert(path.to_lowercase());
            }
            ((folder, id), path)
        };
        let mut paths = HashMap::new();
        for a in &snap.areas {
            paths.extend([place(AREAS, a.archived_at.is_some(), &a.name, a.id)]);
        }
        for p in &snap.projects {
            paths.extend([place(PROJECTS, p.archived_at.is_some(), &p.name, p.id)]);
        }
        for r in &snap.resources {
            paths.extend([place(RESOURCES, r.archived_at.is_some(), &r.title, r.id)]);
        }
        snap.paths = paths;
        Ok(snap)
    }

    fn path(&self, folder: &'static str, id: Option<i64>) -> Option<&String> {
        self.paths.get(&(folder, id?))
    }

    fn area(&self, a: &areas::Model) -> String {
        let mut md = front("area", a.id, &a.name, a.archived_at.is_some());
        heading(&mut md, &a.name, a.description.as_deref());
        let projects = self.projects.iter().filter(|p| p.area_id == a.id);
        section(
            &mut md,
            "Projects",
            projects.filter_map(|p| {
                let status = p.status.as_deref().unwrap_or(projects::DEFAULT_STATUS);
                self.path(PROJECTS, Some(p.id))
                    .map(|path| format!("{} ({status})", link(path, &p.name)))
            }),
        );
        let loose = self
            .resources
            .iter()
            .filter(|r| r.area_id == Some(a.id) && r.project_id.is_none());
        section(
            &mut md,
            "Resources",
            loose.filter_map(|r| self.path(RESOURCES, Some(r.id)).map(|p| link(p, &r.title))),
        );
        md
    }

    fn project(&self, p: &projects::Model, pulled: &[(notes::Model, Option<String>)]) -> String {
        let mut md = front("project", p.id, &p.name, p.archived_at.is_some());
        if let Some(path) = self.path(AREAS, Some(p.area_id)) {
            let _ = writeln!(md, "area: {}", yaml(&format!("[[{path}]]")));
        }
        let status = p.status.as_deref().unwrap_or(projects::DEFAULT_STATUS);
        let _ = writeln!(md, "status: {status}");
        if let Some(due) = p.due_on {
            let _ = writeln!(md, "due: {due}");
        }
        heading(&mut md, &p.name, p.description.as_deref());
        let tasks = self.tasks.iter().filter(|t| t.project_id == p.id);
        section(
            &mut md,
            "Tasks",
            tasks.map(|t| {
                let due = t.due_on.map(|d| format!(" 📅 {d}")).unwrap_or_default();
                format!("[{}] {}{due}", if t.done { "x" } else { " " }, t.title)
            }),
        );
        section(
            &mut md,
            "Notes",
            pulled.iter().map(|(n, why)| {
                let why = why.as_deref().map(|w| format!(": {w}")).unwrap_or_default();
                format!("{}{why}", link(&note_stem(n), &n.zid))
            }),
        );
        let filed = self.resources.iter().filter(|r| r.project_id == Some(p.id));
        section(
            &mut md,
            "Resources",
            filed.filter_map(|r| self.path(RESOURCES, Some(r.id)).map(|x| link(x, &r.title))),
        );
        md
    }

    fn resource(&self, r: &resources::Model) -> String {
        let mut md = front("resource", r.id, &r.title, r.archived_at.is_some());
        for (key, value) in [
            ("kind", r.kind.as_deref()),
            ("url", r.url.as_deref()),
            ("citekey", r.citekey.as_deref()),
            ("external_id", r.external_id.as_deref()),
        ] {
            if let Some(v) = value {
                let _ = writeln!(md, "{key}: {}", yaml(v));
            }
        }
        if let Some(path) = self.path(AREAS, r.area_id) {
            let _ = writeln!(md, "area: {}", yaml(&format!("[[{path}]]")));
        }
        if let Some(path) = self.path(PROJECTS, r.project_id) {
            let _ = writeln!(md, "project: {}", yaml(&format!("[[{path}]]")));
        }
        heading(&mut md, &r.title, r.body.as_deref());
        let lit = self.notes.iter().filter(|n| n.resource_id == Some(r.id));
        section(
            &mut md,
            "Literature notes",
            lit.map(|n| link(&note_stem(n), &n.title)),
        );
        md
    }

    /// A note, with every body link that ztk resolves (zid, then title)
    /// pointing at the target's file stem.
    fn note(&self, n: &notes::Model, tags: &[String], links: &[Linked]) -> String {
        let mut md = String::from("---\nztk: note\n");
        let _ = writeln!(md, "zid: {}", yaml(&n.zid));
        let _ = writeln!(md, "title: {}", yaml(&n.title));
        let _ = writeln!(md, "aliases: [{}]", yaml(&n.title));
        let _ = writeln!(md, "kind: {}", n.kind);
        if let Some(source) = &n.source {
            let _ = writeln!(md, "source: {}", yaml(source));
        }
        if let Some(path) = self.path(RESOURCES, n.resource_id) {
            let _ = writeln!(md, "resource: {}", yaml(&format!("[[{path}]]")));
        }
        if let Some(name) = n.hippo_id.and_then(|id| self.hippos.get(&id)) {
            let _ = writeln!(md, "hippo: {}", yaml(name));
        }
        if !tags.is_empty() {
            let tags: Vec<String> = tags.iter().map(|t| yaml(t)).collect();
            let _ = writeln!(md, "tags: [{}]", tags.join(", "));
        }
        let _ = writeln!(md, "created: {}", n.created_at.to_rfc3339());
        let _ = writeln!(md, "updated: {}", n.updated_at.to_rfc3339());
        let body = rewrite_wikilinks_with(&n.body, |target, anchor, label| {
            let to = self.notes.iter().find(|m| m.zid == target).or_else(|| {
                self.notes
                    .iter()
                    .find(|m| m.title.eq_ignore_ascii_case(target))
            })?;
            let label = if label.is_empty() {
                format!("|{target}")
            } else {
                label.to_string()
            };
            Some(format!("{}{anchor}{label}", note_stem(to)))
        });
        heading(&mut md, &n.title, Some(&body));
        let manual = links.iter().filter(|l| !l.auto);
        section(
            &mut md,
            "Links",
            manual.map(|l| {
                let why = l
                    .context
                    .as_deref()
                    .map(|c| format!(": {c}"))
                    .unwrap_or_default();
                format!(
                    "{} {}{why}",
                    link(&note_stem(&l.note), &l.note.zid),
                    l.note.title
                )
            }),
        );
        md
    }

    async fn render(&self, db: &DatabaseConnection) -> Result<BTreeMap<PathBuf, String>> {
        let mut files = BTreeMap::new();
        let mut put = |path: &String, md: String| {
            files.insert(PathBuf::from(format!("{path}.md")), md);
        };
        for a in &self.areas {
            put(&self.paths[&(AREAS, a.id)], self.area(a));
        }
        for p in &self.projects {
            let pulled = p.notes(db).await?;
            put(&self.paths[&(PROJECTS, p.id)], self.project(p, &pulled));
        }
        for r in &self.resources {
            put(&self.paths[&(RESOURCES, r.id)], self.resource(r));
        }
        let ids: Vec<i64> = self.notes.iter().map(|n| n.id).collect();
        let tags = notes::Entity::tags_of(db, &ids).await?;
        for n in &self.notes {
            let tags = tags.get(&n.id).map_or(&[][..], Vec::as_slice);
            let md = self.note(n, tags, &n.links(db).await?);
            put(&format!("{NOTES}/{}", note_stem(n)), md);
        }
        Ok(files)
    }
}

/// Write the whole brain into `dir` as an Obsidian vault. Archived PARA items
/// go under `Archive/` only with `archived`.
pub async fn export(db: &DatabaseConnection, dir: &Path, archived: bool) -> Result<ExportReport> {
    let io = |p: &Path, e: std::io::Error| Error::Message(format!("{}: {e}", p.display()));
    let snap = Snapshot::load(db, archived).await?;
    let files = snap.render(db).await?;

    // replace our own files, and only those
    let stale: Vec<PathBuf> = managed_files(dir)
        .into_iter()
        .filter(|p| !files.contains_key(p.strip_prefix(dir).unwrap_or(p)))
        .collect();
    for (rel, md) in &files {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        std::fs::write(&path, md).map_err(|e| io(&path, e))?;
    }
    for path in &stale {
        std::fs::remove_file(path).map_err(|e| io(path, e))?;
    }
    Ok(ExportReport {
        notes: snap.notes.len(),
        areas: snap.areas.len(),
        projects: snap.projects.len(),
        resources: snap.resources.len(),
        removed: stale.len(),
    })
}

/// Markdown files under `dir` that ztk wrote (they carry a `ztk:` key).
fn managed_files(dir: &Path) -> Vec<PathBuf> {
    markdown_files(dir)
        .into_iter()
        .filter(|p| {
            std::fs::read_to_string(p)
                .ok()
                .and_then(|text| split_front(&text).0)
                .is_some_and(|fm| fm.get("ztk").is_some())
        })
        .collect()
}

fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue; // .obsidian, .trash, .git
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// in

/// Split `---` front matter from the rest. A file without it has none.
fn split_front(text: &str) -> (Option<Value>, &str) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(rest) = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
    else {
        return (None, text);
    };
    let end = rest.match_indices("\n---").map(|(i, _)| i).find(|&i| {
        let after = &rest[i + 4..];
        after.is_empty() || after.starts_with('\n') || after.starts_with("\r\n")
    });
    let Some(end) = end else {
        return (None, text);
    };
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']);
    let fm = serde_yaml::from_str::<Value>(&rest[..end])
        .ok()
        .filter(Value::is_mapping);
    (fm, body)
}

fn get_str(fm: Option<&Value>, key: &str) -> Option<String> {
    match fm?.get(key)? {
        Value::String(s) => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
    .filter(|s| !s.is_empty())
}

/// `tags: [a, b]`, `tags: a, b`, `tags:\n - a` — Obsidian accepts them all.
fn get_tags(fm: Option<&Value>) -> Vec<String> {
    let raw: Vec<String> = match fm.and_then(|f| f.get("tags")) {
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(|i| i.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(s)) => s.split([',', ' ']).map(str::to_string).collect(),
        _ => Vec::new(),
    };
    raw.into_iter()
        .map(|t| t.trim().trim_start_matches('#').to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// `[[Areas/Health]]` or `[[Areas/Health|Health]]` or `Health` → `Health`.
fn link_name(v: &str) -> String {
    let v = v.trim().trim_start_matches("[[").trim_end_matches("]]");
    let v = v.split('|').next().unwrap_or(v);
    v.rsplit('/').next().unwrap_or(v).trim().to_string()
}

/// Drop a leading `# Title` that just repeats the title.
fn strip_title<'a>(body: &'a str, title: &str) -> &'a str {
    if let Some(rest) = body.trim_start().strip_prefix("# ") {
        let (line, after) = rest.split_once('\n').unwrap_or((rest, ""));
        if line.trim() == title.trim() {
            return after.trim_start_matches(['\r', '\n']);
        }
    }
    body
}

/// The list sections ztk writes at the end of a file (`## Links`, `## Tasks`,
/// ...): cut them off the body and return their items by heading. A section
/// that is not only a list is the person's own prose and stays.
fn cut_sections<'a>(body: &'a str, headings: &[&str]) -> (&'a str, HashMap<String, Vec<String>>) {
    let mut sections = HashMap::new();
    let mut end = body.len();
    loop {
        let head = &body[..end];
        let start = match head.rfind("\n## ") {
            Some(pos) => pos + 1,
            None if head.starts_with("## ") => 0,
            None => break,
        };
        let block = &head[start + 3..];
        let (name, rest) = block.split_once('\n').unwrap_or((block, ""));
        let items: Vec<&str> = rest
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if !headings.contains(&name.trim()) || !items.iter().all(|l| l.starts_with("- ")) {
            break;
        }
        sections.insert(
            name.trim().to_string(),
            items.iter().map(|l| l[2..].to_string()).collect(),
        );
        end = start;
    }
    (body[..end].trim_end(), sections)
}

/// `[[stem|label]] rest` → (`stem`, `rest`).
fn leading_link(line: &str) -> Option<(String, String)> {
    let (inner, rest) = line.strip_prefix("[[")?.split_once("]]")?;
    let target = inner.split('|').next().unwrap_or(inner).trim().to_string();
    Some((target, rest.trim().to_string()))
}

fn stem_of(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn text_or_none(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// A file of the vault, read but not yet in the brain.
struct Read {
    kind: Option<String>,
    fm: Option<Value>,
    title: String,
    body: String,
    archived: bool,
    stem: String,
}

impl Read {
    fn new(path: &Path, vault: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let (fm, body) = split_front(&text);
        let stem = stem_of(path);
        let h1 = body
            .trim_start()
            .strip_prefix("# ")
            .and_then(|r| r.lines().next())
            .map(|l| l.trim().to_string());
        let zid_less = |zid: Option<String>| {
            zid.map_or_else(
                || stem.clone(),
                |z| stem.strip_prefix(&z).unwrap_or(&stem).trim().to_string(),
            )
        };
        let title = get_str(fm.as_ref(), "title")
            .or(h1)
            .unwrap_or_else(|| zid_less(get_str(fm.as_ref(), "zid")));
        let in_archive = path
            .strip_prefix(vault)
            .ok()
            .and_then(|p| p.components().next())
            .is_some_and(|c| c.as_os_str() == ARCHIVE);
        let archived = in_archive || get_str(fm.as_ref(), "archived").as_deref() == Some("true");
        let body = strip_title(body, &title).to_string();
        Some(Self {
            kind: get_str(fm.as_ref(), "ztk"),
            fm,
            title,
            body,
            archived,
            stem,
        })
    }

    fn get(&self, key: &str) -> Option<String> {
        get_str(self.fm.as_ref(), key)
    }

    fn link(&self, key: &str) -> Option<String> {
        self.get(key).map(|v| link_name(&v))
    }
}

/// Read an Obsidian vault into the brain.
///
/// Files with `ztk: area|project|resource` front matter become PARA items,
/// matched by name, so a re-import updates them; every other Markdown file is
/// a note, matched by its `zid` (or, without one, by its title). With
/// `dry_run` everything runs in a transaction that is rolled back.
pub async fn import(db: &DatabaseConnection, dir: &Path, dry_run: bool) -> Result<ImportReport> {
    if !dir.is_dir() {
        return Err(Error::Message(format!("{} is not a folder", dir.display())));
    }
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    for path in markdown_files(dir) {
        match Read::new(&path, dir) {
            Some(f) => files.push(f),
            None => warnings.push(format!("{}: not UTF-8, skipped", path.display())),
        }
    }
    let txn = db.begin().await?;
    let mut importer = Importer::new(&txn, dry_run);
    importer.report.warnings = warnings;
    let result = importer.run(&files).await;
    let report = importer.report;
    match result {
        Ok(()) if !dry_run => txn.commit().await?,
        Ok(()) => txn.rollback().await?,
        Err(e) => {
            txn.rollback().await?;
            return Err(crate::cli::explain(e));
        }
    }
    Ok(report)
}

/// One import, step by step: areas, projects, resources, notes, then the
/// links between them, then the archive.
struct Importer<'a> {
    db: &'a DatabaseTransaction,
    report: ImportReport,
    areas: HashMap<String, i64>,
    projects: HashMap<String, i64>,
    resources: HashMap<String, i64>,
    /// Obsidian file stem (lowercased) → zid.
    stems: HashMap<String, String>,
    pulls: Vec<(i64, Vec<String>)>,
    /// (zid, Hippo name): notes join their Hippo last, once their own links
    /// are back, so the weave only fills real gaps.
    joins: Vec<(String, String)>,
    archive: Vec<(&'static str, i64)>,
}

impl<'a> Importer<'a> {
    fn new(db: &'a DatabaseTransaction, dry_run: bool) -> Self {
        Self {
            db,
            report: ImportReport {
                dry_run,
                ..ImportReport::default()
            },
            areas: HashMap::new(),
            projects: HashMap::new(),
            resources: HashMap::new(),
            stems: HashMap::new(),
            pulls: Vec::new(),
            joins: Vec::new(),
            archive: Vec::new(),
        }
    }

    async fn run(&mut self, files: &[Read]) -> ModelResult<()> {
        let of = |kind: &'static str| {
            files
                .iter()
                .filter(move |f| f.kind.as_deref() == Some(kind))
        };
        for a in areas::Entity::find().all(self.db).await? {
            self.areas.insert(a.name.to_lowercase(), a.id);
        }
        for p in projects::Entity::find().all(self.db).await? {
            self.projects.insert(p.name.to_lowercase(), p.id);
        }
        for f in of("area") {
            self.area(f).await?;
        }
        for f in of("project") {
            self.project(f).await?;
        }
        for f in of("resource") {
            self.resource(f).await?;
        }
        // notes that carry a zid go first, so a note without one is never
        // filed under a zid the vault still claims
        let mut notes: Vec<&Read> = files
            .iter()
            .filter(|f| !matches!(f.kind.as_deref(), Some("area" | "project" | "resource")))
            .collect();
        notes.sort_by_key(|f| f.get("zid").is_none());
        let mut imported = Vec::new();
        for f in notes {
            imported.push((self.note(f).await?, f));
        }
        for (note, f) in imported {
            self.note_links(note, f).await?;
        }
        self.pull_notes().await?;
        self.join_hippos().await?;
        // archive last: nothing new may go into an archived parent
        for (kind, id) in std::mem::take(&mut self.archive).into_iter().rev() {
            let db = self.db;
            match kind {
                "area" => {
                    areas::Model::find(db, id)
                        .await?
                        .set_archived(db, true)
                        .await?;
                }
                "project" => {
                    projects::Model::find(db, id)
                        .await?
                        .set_archived(db, true)
                        .await?;
                }
                _ => {
                    resources::Model::find(db, id)
                        .await?
                        .set_archived(db, true)
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn area(&mut self, f: &Read) -> ModelResult<()> {
        let (text, _) = cut_sections(&f.body, &["Projects", "Resources"]);
        let description = text_or_none(text);
        let id = if let Some(&id) = self.areas.get(&f.title.to_lowercase()) {
            // reopen it so its projects can be updated; archived again at the end
            let area = areas::Model::find(self.db, id).await?;
            let area = if area.archived_at.is_some() {
                area.set_archived(self.db, false).await?
            } else {
                area
            };
            let params = UpdateArea {
                name: None,
                description: description.into(),
            };
            area.update_with(self.db, params).await?.id
        } else {
            let params = CreateArea {
                name: f.title.clone(),
                description,
            };
            let area = areas::ActiveModel::create(self.db, params).await?;
            self.areas.insert(area.name.to_lowercase(), area.id);
            area.id
        };
        if f.archived {
            self.archive.push(("area", id));
        }
        self.report.areas += 1;
        Ok(())
    }

    /// The area a project goes into: the named one, or "Inbox" (made if needed).
    async fn area_for(&mut self, f: &Read) -> ModelResult<i64> {
        if let Some(&id) = f
            .link("area")
            .and_then(|a| self.areas.get(&a.to_lowercase()))
        {
            return Ok(id);
        }
        self.report.warnings.push(format!(
            "project \"{}\": no known area, filed under Inbox",
            f.title
        ));
        if let Some(&id) = self.areas.get("inbox") {
            return Ok(id);
        }
        let params = CreateArea {
            name: "Inbox".into(),
            description: None,
        };
        let area = areas::ActiveModel::create(self.db, params).await?;
        self.areas.insert("inbox".into(), area.id);
        Ok(area.id)
    }

    async fn project(&mut self, f: &Read) -> ModelResult<()> {
        let db = self.db;
        let area_id = self.area_for(f).await?;
        let (text, sections) = cut_sections(&f.body, &["Tasks", "Notes", "Resources"]);
        let description = text_or_none(text);
        let due: Option<Date> = f.get("due").and_then(|d| d.parse().ok());
        let project = if let Some(&id) = self.projects.get(&f.title.to_lowercase()) {
            let mut project = projects::Model::find(db, id).await?;
            if project.archived_at.is_some() {
                // its area may still be archived in the brain
                let area = areas::Model::find(db, project.area_id).await?;
                if area.archived_at.is_some() {
                    area.set_archived(db, false).await?;
                }
                project = project.set_archived(db, false).await?;
            }
            let params = UpdateProject {
                name: None,
                description: description.into(),
                status: f.get("status"),
                area_id: Some(area_id),
                due_on: due.into(),
            };
            project.update_with(db, params).await?
        } else {
            let params = CreateProject {
                name: f.title.clone(),
                description,
                status: f.get("status"),
                area_id,
                due_on: due,
            };
            let project = projects::ActiveModel::create(db, params).await?;
            self.projects
                .insert(project.name.to_lowercase(), project.id);
            project
        };
        let lines = sections.get("Tasks").map_or(&[][..], Vec::as_slice);
        self.tasks(&project, lines).await?;
        let notes = sections.get("Notes").cloned().unwrap_or_default();
        self.pulls.push((project.id, notes));
        if f.archived {
            self.archive.push(("project", project.id));
        }
        self.report.projects += 1;
        Ok(())
    }

    /// `[ ] title 📅 2026-11-01` and `[x] title`, matched by title.
    async fn tasks(&mut self, project: &projects::Model, lines: &[String]) -> ModelResult<()> {
        let existing = tasks::Entity::listing(Some(project.id), None)
            .all(self.db)
            .await?;
        for line in lines {
            let parsed = line
                .strip_prefix("[ ] ")
                .map(|r| (false, r))
                .or_else(|| line.strip_prefix("[x] ").map(|r| (true, r)))
                .or_else(|| line.strip_prefix("[X] ").map(|r| (true, r)));
            let Some((done, rest)) = parsed else {
                let name = &project.name;
                self.report
                    .warnings
                    .push(format!("project \"{name}\": not a task line: {line}"));
                continue;
            };
            let (title, due_on) = rest.rsplit_once(" 📅 ").map_or_else(
                || (rest.trim(), None),
                |(t, d)| (t.trim(), d.trim().parse::<Date>().ok()),
            );
            if let Some(t) = existing.iter().find(|t| t.title == title) {
                let params = UpdateTask {
                    done: Some(done),
                    due_on: due_on.into(),
                    ..Default::default()
                };
                t.clone().update_with(self.db, params).await?;
            } else {
                let params = CreateTask {
                    title: title.to_string(),
                    notes: None,
                    done,
                    due_on,
                    project_id: project.id,
                };
                tasks::ActiveModel::create(self.db, params).await?;
            }
            self.report.tasks += 1;
        }
        Ok(())
    }

    /// The resource a file names: by `external_id`, then `citekey`, then title.
    async fn existing_resource(&self, f: &Read) -> ModelResult<Option<resources::Model>> {
        if let Some(ext) = f.get("external_id") {
            if let Some(r) = resources::Entity::by_external_id(self.db, &ext).await? {
                return Ok(Some(r));
            }
        }
        if let Some(key) = f.get("citekey") {
            let key = key.trim_start_matches('@').to_string();
            let q = resources::Entity::find().filter(resources::Column::Citekey.eq(key));
            if let Some(r) = q.one(self.db).await? {
                return Ok(Some(r));
            }
        }
        let q = resources::Entity::find().filter(resources::Column::Title.eq(f.title.as_str()));
        Ok(q.one(self.db).await?)
    }

    async fn resource(&mut self, f: &Read) -> ModelResult<()> {
        let db = self.db;
        let (text, _) = cut_sections(&f.body, &["Literature notes"]);
        let body = text_or_none(text);
        let project_id = f
            .link("project")
            .and_then(|p| self.projects.get(&p.to_lowercase()).copied());
        let mut area_id = f
            .link("area")
            .and_then(|a| self.areas.get(&a.to_lowercase()).copied());
        // the project decides the area when both are known
        if let (Some(_), Some(pid)) = (area_id, project_id) {
            area_id = Some(projects::Model::find(db, pid).await?.area_id);
        }
        let r = if let Some(r) = self.existing_resource(f).await? {
            let params = UpdateResource {
                title: Some(f.title.clone()),
                url: f.get("url").into(),
                body: body.into(),
                area_id: area_id.into(),
                project_id: project_id.into(),
                kind: f.get("kind").into(),
                citekey: Patch::Keep,
            };
            r.update_with(db, params).await?
        } else {
            let params = CreateResource {
                title: f.title.clone(),
                url: f.get("url"),
                body,
                area_id,
                project_id,
                kind: f.get("kind"),
                citekey: f.get("citekey"),
                external_id: f.get("external_id"),
            };
            resources::ActiveModel::create(db, params).await?
        };
        self.resources.insert(r.title.to_lowercase(), r.id);
        if f.archived {
            self.archive.push(("resource", r.id));
        }
        self.report.resources += 1;
        Ok(())
    }

    /// The note's kind, made to fit the method: unknown kinds and literature
    /// notes with nothing to cite come in as fleeting, with a warning.
    fn kind(&mut self, f: &Read, sourced: bool) -> String {
        let raw = f
            .get("kind")
            .or_else(|| f.get("type"))
            .unwrap_or_else(|| notes::DEFAULT_KIND.to_string())
            .to_lowercase();
        let kind = match raw.as_str() {
            "structure" | "moc" | "index" => "hub".to_string(),
            _ => raw,
        };
        if !notes::KINDS.contains(&kind.as_str()) {
            self.report.warnings.push(format!(
                "{}: kind \"{kind}\" is not a ztk kind, imported as fleeting",
                f.stem
            ));
            return notes::DEFAULT_KIND.into();
        }
        if kind == "literature" && !sourced {
            self.report.warnings.push(format!(
                "{}: literature note without a source, imported as fleeting",
                f.stem
            ));
            return notes::DEFAULT_KIND.into();
        }
        kind
    }

    async fn note(&mut self, f: &Read) -> ModelResult<notes::Model> {
        let db = self.db;
        let zid = f.get("zid").filter(|z| notes::is_zid(z));
        let resource_id = f
            .link("resource")
            .and_then(|r| self.resources.get(&r.to_lowercase()).copied());
        let source = f.get("source");
        let kind = self.kind(f, source.is_some() || resource_id.is_some());
        let (text, _) = cut_sections(&f.body, &["Links"]);
        let tags = get_tags(f.fm.as_ref());
        // by zid; a note of the person's own (no zid) by its title, so a
        // re-import updates it instead of filing it twice
        let existing = match &zid {
            Some(z) => notes::Model::find(db, z).await.ok(),
            None => notes::Model::resolve(db, &f.title)
                .await?
                .filter(|n| n.title.eq_ignore_ascii_case(&f.title)),
        };
        let note = if let Some(note) = existing {
            self.report.notes_updated += 1;
            let params = UpdateNote {
                title: Some(f.title.clone()),
                body: Some(text.to_string()),
                kind: Some(kind),
                source: source.into(),
                resource_id: resource_id.into(),
                hippo_id: Patch::Keep,
                tags: Some(tags),
            };
            note.update_with(db, params).await?
        } else {
            self.report.notes_created += 1;
            let params = CreateNote {
                title: f.title.clone(),
                body: text.to_string(),
                kind: Some(kind),
                source,
                resource_id,
                hippo_id: None,
                after: None,
                tags: Some(tags),
                zid,
            };
            notes::ActiveModel::create(db, params).await?
        };
        self.stems.insert(f.stem.to_lowercase(), note.zid.clone());
        if let Some(name) = f.get("hippo") {
            self.joins.push((note.zid.clone(), name));
        }
        Ok(note)
    }

    /// Put notes back in their Hippo's rhizome, making the Hippo if the brain
    /// has none by that name.
    async fn join_hippos(&mut self) -> ModelResult<()> {
        let db = self.db;
        for (zid, name) in std::mem::take(&mut self.joins) {
            let found = hippos::Entity::find()
                .filter(hippos::Column::Name.eq(name.as_str()))
                .one(db)
                .await?;
            let hippo = if let Some(h) = found {
                h
            } else {
                self.report.warnings.push(format!(
                    "Hippo \"{name}\" created; set its traits and fixations with ztk hippo -u"
                ));
                let params = crate::dtos::hippos::CreateHippo {
                    name: name.clone(),
                    ..Default::default()
                };
                hippos::ActiveModel::create(db, params).await?
            };
            let params = UpdateNote {
                hippo_id: Patch::Set(hippo.id),
                ..Default::default()
            };
            notes::Model::find(db, &zid)
                .await?
                .update_with(db, params)
                .await?;
        }
        Ok(())
    }

    /// The note a vault link points at: a file stem we imported, or a zid.
    async fn target(&self, stem: &str) -> ModelResult<Option<notes::Model>> {
        let zid = self
            .stems
            .get(&stem.to_lowercase())
            .cloned()
            .or_else(|| notes::is_zid(stem).then(|| stem.to_string()));
        match zid {
            Some(z) => Ok(notes::Model::find(self.db, &z).await.ok()),
            None => Ok(None),
        }
    }

    /// Point body links at zids, then restore the hand-made links with their reasons.
    async fn note_links(&mut self, note: notes::Model, f: &Read) -> ModelResult<()> {
        let db = self.db;
        let body = rewrite_wikilinks_with(&note.body, |target, anchor, label| {
            let zid = self.stems.get(&target.to_lowercase())?;
            let label = if label == format!("|{zid}") {
                ""
            } else {
                label
            };
            Some(format!("{zid}{anchor}{label}"))
        });
        let note = if body == note.body {
            note
        } else {
            let params = UpdateNote {
                body: Some(body),
                ..Default::default()
            };
            note.update_with(db, params).await?
        };
        let (_, sections) = cut_sections(&f.body, &["Links"]);
        for line in sections.get("Links").map_or(&[][..], Vec::as_slice) {
            let found = match leading_link(line) {
                Some((stem, rest)) => self.target(&stem).await?.map(|t| (t, rest)),
                None => None,
            };
            let Some((target, rest)) = found else {
                let stem = &f.stem;
                self.report
                    .warnings
                    .push(format!("{stem}: link to an unknown note: {line}"));
                continue;
            };
            // `Title: why` → why
            let why = rest.strip_prefix(target.title.as_str()).unwrap_or(&rest);
            let why = why.trim_start().strip_prefix(':').map(str::trim);
            let why = why.filter(|w| !w.is_empty()).map(str::to_string);
            note.link_to(db, &target, why).await?;
            self.report.links += 1;
        }
        Ok(())
    }

    async fn pull_notes(&mut self) -> ModelResult<()> {
        for (project_id, lines) in std::mem::take(&mut self.pulls) {
            let project = projects::Model::find(self.db, project_id).await?;
            for line in lines {
                let found = match leading_link(&line) {
                    Some((stem, rest)) => self.target(&stem).await?.map(|t| (t, rest)),
                    None => None,
                };
                let Some((note, rest)) = found else {
                    let name = &project.name;
                    self.report
                        .warnings
                        .push(format!("project \"{name}\": unknown note: {line}"));
                    continue;
                };
                let why = rest.trim_start().strip_prefix(':').map(str::trim);
                let why = why.filter(|w| !w.is_empty()).map(str::to_string);
                project.link_note(self.db, &note, why).await?;
                self.report.links += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter_and_sections() {
        let text = "---\nztk: note\ntags: [a, \"#b\"]\n---\n\n# T\n\nbody\n\n## Links\n\n- [[1a X|1a]] X: because\n";
        let (fm, body) = split_front(text);
        assert_eq!(get_str(fm.as_ref(), "ztk").as_deref(), Some("note"));
        assert_eq!(get_tags(fm.as_ref()), ["a", "b"]);
        let (text, sections) = cut_sections(strip_title(body, "T"), &["Links"]);
        assert_eq!(text, "body");
        assert_eq!(sections["Links"], ["[[1a X|1a]] X: because"]);
        // a section of the person's own prose is kept
        let own = "intro\n\n## Links\n\nsome prose";
        let (text, sections) = cut_sections(own, &["Links"]);
        assert_eq!(text, own);
        assert!(sections.is_empty());
        assert_eq!(split_front("no front matter").1, "no front matter");
    }

    #[test]
    fn names_and_links() {
        assert_eq!(safe("a/b: c?"), "ab c");
        assert_eq!(safe("..."), "untitled");
        assert_eq!(link_name("[[Archive/Areas/Health|Health]]"), "Health");
        assert_eq!(
            leading_link("[[1a X|1a]] X: why"),
            Some(("1a X".into(), "X: why".into()))
        );
    }
}
