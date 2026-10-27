#![allow(clippy::missing_errors_doc)]
use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

use crate::dtos::common::Patch;

use loco_rs::model::ModelResult;
use sea_orm::ConnectionTrait;

use crate::models::notes::{Entity, Linked, Model};

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct NoteDto {
    #[ts(type = "number")]
    pub id: i64,
    /// Slipbox id (`21a3`): where the note sits in its train of thought.
    pub zid: String,
    pub title: String,
    pub body: String,
    /// `fleeting`, `literature`, `permanent` or `hub`.
    pub kind: String,
    pub source: Option<String>,
    /// The resource a literature note was written from.
    #[ts(type = "number | null")]
    pub resource_id: Option<i64>,
    /// The Hippo identity whose rhizome the note belongs to.
    #[ts(type = "number | null")]
    pub hippo_id: Option<i64>,
    pub tags: Vec<String>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl NoteDto {
    #[must_use]
    pub fn new(m: Model, tags: Vec<String>) -> Self {
        Self {
            id: m.id,
            zid: m.zid,
            title: m.title,
            body: m.body,
            kind: m.kind,
            source: m.source,
            resource_id: m.resource_id,
            hippo_id: m.hippo_id,
            tags,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl NoteDto {
    /// Render many notes, loading their tags in one query.
    pub async fn many<C: ConnectionTrait>(db: &C, notes: Vec<Model>) -> ModelResult<Vec<Self>> {
        let ids: Vec<i64> = notes.iter().map(|n| n.id).collect();
        let mut tags = Entity::tags_of(db, &ids).await?;
        Ok(notes
            .into_iter()
            .map(|n| {
                let t = tags.remove(&n.id).unwrap_or_default();
                Self::new(n, t)
            })
            .collect())
    }

    pub async fn one<C: ConnectionTrait>(db: &C, note: Model) -> ModelResult<Self> {
        let tags = note.tags(db).await?;
        Ok(Self::new(note, tags))
    }
}

impl NoteDetail {
    pub async fn load<C: ConnectionTrait>(db: &C, note: Model) -> ModelResult<Self> {
        let links = note.links(db).await?.into_iter().map(Into::into).collect();
        let backlinks = note
            .backlinks(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Self {
            note: NoteDto::one(db, note).await?,
            links,
            backlinks,
        })
    }
}

impl From<&Model> for TreeEntry {
    fn from(n: &Model) -> Self {
        Self {
            zid: n.zid.clone(),
            title: n.title.clone(),
            kind: n.kind.clone(),
            depth: n.depth(),
        }
    }
}

impl GraphDto {
    pub async fn load<C: ConnectionTrait>(db: &C) -> ModelResult<Self> {
        let nodes = Entity::tree(db)
            .await?
            .into_iter()
            .map(|n| GraphNode {
                zid: n.zid,
                title: n.title,
                kind: n.kind,
            })
            .collect();
        let edges = Entity::edges(db)
            .await?
            .into_iter()
            .map(|(source, target)| GraphEdge { source, target })
            .collect();
        Ok(Self { nodes, edges })
    }
}

/// The other end of a link.
#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct LinkDto {
    pub zid: String,
    pub title: String,
    pub kind: String,
    /// Why the two notes belong together (hand-made links only).
    pub context: Option<String>,
    /// True when the link comes from a `[[wikilink]]` in the body.
    pub auto: bool,
}

impl From<Linked> for LinkDto {
    fn from(l: Linked) -> Self {
        Self {
            zid: l.note.zid,
            title: l.note.title,
            kind: l.note.kind,
            context: l.context,
            auto: l.auto,
        }
    }
}

/// One note with everything around it.
#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct NoteDetail {
    #[serde(flatten)]
    pub note: NoteDto,
    pub links: Vec<LinkDto>,
    pub backlinks: Vec<LinkDto>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateNote {
    pub title: String,
    #[serde(default)]
    pub body: String,
    /// Defaults to `fleeting`.
    pub kind: Option<String>,
    /// Required for `literature`, unless `resource_id` is given.
    pub source: Option<String>,
    /// The resource a literature note was written from; its citation becomes the
    /// `source` when none is given.
    #[ts(type = "number | null")]
    pub resource_id: Option<i64>,
    /// Grow the note in this Hippo's rhizome: it is linked into the network on
    /// save, and `after` is refused (a rhizome has no trunk).
    #[ts(type = "number | null")]
    pub hippo_id: Option<i64>,
    /// File the note as a continuation of this zid (`21` → `21a`).
    pub after: Option<String>,
    /// Added to the `#hashtags` found in the body.
    pub tags: Option<Vec<String>>,
    /// Keep this zid instead of filing the note (importers only; must be free).
    #[serde(skip)]
    #[ts(skip)]
    pub zid: Option<String>,
}

/// Partial update: only the fields that are present are changed.
/// `tags`, when present, replaces the explicit tags (`#hashtags` in the body always count);
/// `null` clears `source` or `resource_id`.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub body: Option<String>,
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub source: Patch<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "number | null")]
    pub resource_id: Patch<i64>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "number | null")]
    pub hippo_id: Patch<i64>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateLink {
    /// zid of the note to link to.
    pub target: String,
    /// Why the two notes belong together.
    pub context: Option<String>,
}

/// A line of the slipbox tree: notes in Folgezettel order, indented by depth.
#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct TreeEntry {
    pub zid: String,
    pub title: String,
    pub kind: String,
    #[ts(type = "number")]
    pub depth: usize,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct GraphNode {
    pub zid: String,
    pub title: String,
    pub kind: String,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

/// The whole slipbox as a graph, for visualizers.
#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct GraphDto {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct TagCount {
    pub name: String,
    #[ts(type = "number")]
    pub count: i64,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct StatsDto {
    #[ts(type = "number")]
    pub notes: u64,
    #[ts(type = "number")]
    pub links: u64,
    #[ts(type = "number")]
    pub inbox: u64,
    #[ts(type = "number")]
    pub orphans: u64,
    pub by_kind: Vec<TagCount>,
    pub tags: Vec<TagCount>,
}
