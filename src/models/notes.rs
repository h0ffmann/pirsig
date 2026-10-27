//! Notes are the slipbox.
//!
//! Zettelkasten cards with Luhmann-style ids (`1`, `1a`, `1a1`, ...), `[[wikilinks]]` that turn into links on save, and
//! `#hashtags` that turn into tags. Model rules shared by the REST controllers
//! and the `ztk` CLI.
#![allow(clippy::missing_errors_doc)]
use std::{cmp::Ordering, collections::HashMap, fmt::Write};

pub use super::_entities::notes::{ActiveModel, Column, Entity, Model};
use loco_rs::{
    model::{ModelError, ModelResult},
    prelude::{Validatable, Validate},
};
use sea_orm::{
    entity::prelude::*,
    sea_query::{ExprTrait, Func, LikeExpr, Query, SimpleExpr},
    ActiveValue::Set,
    Condition, IntoActiveModel, PaginatorTrait, QueryOrder, QuerySelect, TransactionSession,
    TransactionTrait,
};
use serde::Deserialize;

use super::{
    hippos, note_links, note_tags, project_notes, resources,
    rules::{self, invalid},
};
use crate::dtos::notes::{CreateNote, StatsDto, TagCount, UpdateNote};
pub type Notes = Entity;

/// Note kinds, after Ahrens' "How to Take Smart Notes":
/// quick captures, notes on a source, notes in your own words, and
/// structure notes that index a topic.
pub const KINDS: [&str; 4] = ["fleeting", "literature", "permanent", "hub"];
pub const DEFAULT_KIND: &str = "fleeting";

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 1, message = "title cannot be empty"))]
    pub title: String,
    #[validate(custom(function = "validate_kind"))]
    pub kind: String,
    #[validate(custom(function = "validate_zid"))]
    pub zid: String,
}

fn validate_kind(kind: &str) -> Result<(), validator::ValidationError> {
    if KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("kind")
            .with_message(format!("kind must be one of {}", KINDS.join(", ")).into()))
    }
}

fn validate_zid(zid: &str) -> Result<(), validator::ValidationError> {
    if is_zid(zid) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("zid").with_message(
            "zid looks like 1, 1a, 1a2 (digits and lowercase letters, alternating)".into(),
        ))
    }
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            title: self.title.as_ref().clone(),
            kind: self.kind.as_ref().clone(),
            zid: self.zid.as_ref().clone(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.validate()?;
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

/// A link as seen from one note: the note on the other end and why they connect.
#[derive(Debug, Clone)]
pub struct Linked {
    pub note: Model,
    pub context: Option<String>,
    pub auto: bool,
}

// implement your read-oriented logic here
impl Model {
    /// Load a note by its slipbox id (`21a3`) or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, zid: &str) -> ModelResult<Self> {
        Entity::find()
            .filter(Column::Zid.eq(zid))
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// Resolve a `[[wikilink]]` target: a zid first, then a title (case-insensitive).
    pub async fn resolve<C: ConnectionTrait>(db: &C, key: &str) -> ModelResult<Option<Self>> {
        if let Some(note) = Entity::find().filter(Column::Zid.eq(key)).one(db).await? {
            return Ok(Some(note));
        }
        Ok(Entity::find()
            .filter(ExprTrait::eq(lower(Column::Title), key.to_lowercase()))
            .order_by_asc(Column::Id)
            .one(db)
            .await?)
    }

    pub async fn tags<C: ConnectionTrait>(&self, db: &C) -> ModelResult<Vec<String>> {
        Ok(Entity::tags_of(db, &[self.id])
            .await?
            .remove(&self.id)
            .unwrap_or_default())
    }

    /// Notes this one points to.
    pub async fn links<C: ConnectionTrait>(&self, db: &C) -> ModelResult<Vec<Linked>> {
        let rows = note_links::Entity::find()
            .filter(note_links::Column::SourceId.eq(self.id))
            .order_by_asc(note_links::Column::Id)
            .all(db)
            .await?;
        Self::linked(db, rows, |l| l.target_id).await
    }

    /// Notes that point to this one.
    pub async fn backlinks<C: ConnectionTrait>(&self, db: &C) -> ModelResult<Vec<Linked>> {
        let rows = note_links::Entity::find()
            .filter(note_links::Column::TargetId.eq(self.id))
            .order_by_asc(note_links::Column::Id)
            .all(db)
            .await?;
        Self::linked(db, rows, |l| l.source_id).await
    }

    async fn linked<C: ConnectionTrait>(
        db: &C,
        rows: Vec<note_links::Model>,
        other: impl Fn(&note_links::Model) -> i64,
    ) -> ModelResult<Vec<Linked>> {
        let ids: Vec<i64> = rows.iter().map(&other).collect();
        let notes: HashMap<i64, Self> = Entity::find()
            .filter(Column::Id.is_in(ids))
            .all(db)
            .await?
            .into_iter()
            .map(|n| (n.id, n))
            .collect();
        Ok(rows
            .into_iter()
            .filter_map(|l| {
                notes.get(&other(&l)).map(|note| Linked {
                    note: note.clone(),
                    context: l.context,
                    auto: l.auto,
                })
            })
            .collect())
    }

    pub async fn update_with<C: ConnectionTrait + TransactionTrait>(
        self,
        db: &C,
        params: UpdateNote,
    ) -> ModelResult<Self> {
        let txn = db.begin().await?;
        let old_title = self.title.clone();
        let mut item = self.into_active_model();
        if let Some(title) = params.title {
            item.title = Set(rules::text("title", &title)?);
        }
        if let Some(body) = params.body {
            item.body = Set(body);
        }
        if let Some(kind) = params.kind {
            item.kind = Set(normalize_kind(&kind));
        }
        if let Some(source) = params.source.into_option() {
            item.source = Set(rules::optional(source));
        }
        if let Some(resource_id) = params.resource_id.into_option() {
            item.resource_id = Set(resource_id);
        }
        if let Some(hippo_id) = params.hippo_id.into_option() {
            if let Some(id) = hippo_id {
                hippos::Model::find(&txn, id)
                    .await
                    .map_err(|_| invalid("hippo_id", &format!("hippo #{id} does not exist")))?;
            }
            item.hippo_id = Set(hippo_id);
        }
        sourced(&txn, &mut item).await?;
        item.validate()?;
        let note = item.update(&txn).await?;
        let tags = match params.tags {
            Some(tags) => tags,
            None => note.tags(&txn).await?,
        };
        note.sync(&txn, tags).await?;
        if note.title != old_title {
            note.follow_rename(&txn, &old_title).await?;
            note.adopt_dangling_links(&txn).await?;
        }
        hippos::weave(&txn, &note).await?;
        txn.commit().await?;
        Ok(note)
    }

    /// Delete the note with its tags and every link to or from it.
    pub async fn remove<C: ConnectionTrait + TransactionTrait>(self, db: &C) -> ModelResult<()> {
        let txn = db.begin().await?;
        hippos::heal(&txn, &self).await?;
        note_links::Entity::delete_many()
            .filter(
                Condition::any()
                    .add(note_links::Column::SourceId.eq(self.id))
                    .add(note_links::Column::TargetId.eq(self.id)),
            )
            .exec(&txn)
            .await?;
        note_tags::Entity::delete_many()
            .filter(note_tags::Column::NoteId.eq(self.id))
            .exec(&txn)
            .await?;
        project_notes::Entity::delete_many()
            .filter(project_notes::Column::NoteId.eq(self.id))
            .exec(&txn)
            .await?;
        self.into_active_model().delete(&txn).await?;
        txn.commit().await?;
        Ok(())
    }

    /// Link this note to `target` by hand, saying why. Linking twice updates the reason.
    pub async fn link_to<C: ConnectionTrait>(
        &self,
        db: &C,
        target: &Self,
        context: Option<String>,
    ) -> ModelResult<note_links::Model> {
        if self.id == target.id {
            return Err(invalid("target", "a note cannot link to itself"));
        }
        let existing = note_links::Entity::find()
            .filter(note_links::Column::SourceId.eq(self.id))
            .filter(note_links::Column::TargetId.eq(target.id))
            .one(db)
            .await?;
        let link = match existing {
            Some(link) => {
                let mut link = link.into_active_model();
                link.context = Set(context);
                link.auto = Set(false);
                link.update(db).await?
            }
            None => {
                note_links::ActiveModel {
                    source_id: Set(self.id),
                    target_id: Set(target.id),
                    context: Set(context),
                    auto: Set(false),
                    ..Default::default()
                }
                .insert(db)
                .await?
            }
        };
        Ok(link)
    }

    /// Remove the link from this note to `target`; returns whether there was one.
    pub async fn unlink<C: ConnectionTrait>(&self, db: &C, target: &Self) -> ModelResult<bool> {
        let res = note_links::Entity::delete_many()
            .filter(note_links::Column::SourceId.eq(self.id))
            .filter(note_links::Column::TargetId.eq(target.id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// How deep the note sits in its train of thought: `21` is 0, `21a` is 1, `21a3` is 2.
    #[must_use]
    pub fn depth(&self) -> usize {
        zid_segments(&self.zid).len().saturating_sub(1)
    }

    /// The note as a Markdown file with YAML front matter, ready for any editor or git.
    #[must_use]
    pub fn to_markdown(&self, tags: &[String], links: &[Linked]) -> String {
        let mut out = String::from("---\n");
        let _ = writeln!(out, "zid: \"{}\"", self.zid);
        let _ = writeln!(out, "title: {}", yaml_str(&self.title));
        let _ = writeln!(out, "aliases: [{}]", yaml_str(&self.title));
        let _ = writeln!(out, "kind: {}", self.kind);
        if let Some(source) = &self.source {
            let _ = writeln!(out, "source: {}", yaml_str(source));
        }
        if !tags.is_empty() {
            let tags: Vec<String> = tags.iter().map(|t| yaml_str(t)).collect();
            let _ = writeln!(out, "tags: [{}]", tags.join(", "));
        }
        let _ = writeln!(out, "created: {}", self.created_at.to_rfc3339());
        let _ = writeln!(out, "updated: {}", self.updated_at.to_rfc3339());
        out.push_str("---\n\n");
        let _ = writeln!(out, "# {}\n", self.title);
        out.push_str(self.body.trim_end());
        out.push('\n');
        let manual: Vec<&Linked> = links.iter().filter(|l| !l.auto).collect();
        if !manual.is_empty() {
            out.push_str("\n## Links\n\n");
            for l in manual {
                let why = l
                    .context
                    .as_deref()
                    .map(|c| format!(": {c}"))
                    .unwrap_or_default();
                let _ = writeln!(out, "- [[{}]] {}{why}", l.note.zid, l.note.title);
            }
        }
        out
    }

    /// Recompute what the body implies: tags (explicit plus `#hashtags`) and
    /// auto links (`[[wikilinks]]`). Hand-made links are left alone.
    async fn sync<C: ConnectionTrait>(
        &self,
        db: &C,
        explicit_tags: Vec<String>,
    ) -> ModelResult<()> {
        let mut tags: Vec<String> = explicit_tags
            .iter()
            .map(|t| normalize_tag(t))
            .chain(hashtags(&self.body))
            .filter(|t| !t.is_empty())
            .collect();
        tags.sort();
        tags.dedup();
        note_tags::Entity::delete_many()
            .filter(note_tags::Column::NoteId.eq(self.id))
            .exec(db)
            .await?;
        for name in tags {
            note_tags::ActiveModel {
                note_id: Set(self.id),
                name: Set(name),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        note_links::Entity::delete_many()
            .filter(note_links::Column::SourceId.eq(self.id))
            .filter(note_links::Column::Auto.eq(true))
            .exec(db)
            .await?;
        let manual: Vec<i64> = note_links::Entity::find()
            .select_only()
            .column(note_links::Column::TargetId)
            .filter(note_links::Column::SourceId.eq(self.id))
            .into_tuple()
            .all(db)
            .await?;
        let mut seen = manual;
        for key in wikilinks(&self.body) {
            let Some(target) = Self::resolve(db, &key).await? else {
                continue;
            };
            if target.id == self.id || seen.contains(&target.id) {
                continue;
            }
            seen.push(target.id);
            note_links::ActiveModel {
                source_id: Set(self.id),
                target_id: Set(target.id),
                context: Set(None),
                auto: Set(true),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        Ok(())
    }

    /// After a rename, notes that reached this one by `[[Old title]]` would lose
    /// the link on their next save. Rewrite those wikilinks to the new title, the
    /// way Obsidian does, so the link survives.
    async fn follow_rename<C: ConnectionTrait>(&self, db: &C, old_title: &str) -> ModelResult<()> {
        let old = old_title.to_lowercase();
        for linked in self.backlinks(db).await? {
            let source = linked.note;
            let body = rewrite_wikilinks(&source.body, |target| {
                (target.to_lowercase() == old).then(|| self.title.clone())
            });
            if body != source.body {
                let mut item = source.into_active_model();
                item.body = Set(body);
                item.update(db).await?;
            }
        }
        Ok(())
    }

    /// Notes written before this one may already say `[[its title]]` or
    /// `[[its zid]]`; connect them now that the target exists.
    async fn adopt_dangling_links<C: ConnectionTrait>(&self, db: &C) -> ModelResult<()> {
        let candidates = Entity::find()
            .filter(Column::Id.ne(self.id))
            .filter(
                Condition::any()
                    .add(Column::Body.contains(format!("[[{}", self.zid)))
                    .add(
                        lower(Column::Body)
                            .like(contains(&format!("[[{}", self.title.to_lowercase()))),
                    ),
            )
            .all(db)
            .await?;
        for note in candidates {
            let wanted = wikilinks(&note.body)
                .iter()
                .any(|k| k == &self.zid || k.to_lowercase() == self.title.to_lowercase());
            let exists = note_links::Entity::find()
                .filter(note_links::Column::SourceId.eq(note.id))
                .filter(note_links::Column::TargetId.eq(self.id))
                .one(db)
                .await?
                .is_some();
            if wanted && !exists {
                note_links::ActiveModel {
                    source_id: Set(note.id),
                    target_id: Set(self.id),
                    context: Set(None),
                    auto: Set(true),
                    ..Default::default()
                }
                .insert(db)
                .await?;
            }
        }
        Ok(())
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    /// Create a note. `after` files it as a continuation of that note
    /// (`21` → `21a`, `21a` → `21a1`); without it the note opens a new
    /// top-level train of thought (`22`).
    pub async fn create<C: ConnectionTrait + TransactionTrait>(
        db: &C,
        params: CreateNote,
    ) -> ModelResult<Model> {
        let txn = db.begin().await?;
        if let Some(id) = params.hippo_id {
            hippos::Model::find(&txn, id)
                .await
                .map_err(|_| invalid("hippo_id", &format!("hippo #{id} does not exist")))?;
            if params.after.is_some() {
                return Err(invalid(
                    "after",
                    "a Hippo's notes form a rhizome, with no trunk: link the note instead of filing it under another",
                ));
            }
        }
        let zid = if let Some(zid) = params.zid {
            if Entity::find()
                .filter(Column::Zid.eq(&zid))
                .one(&txn)
                .await?
                .is_some()
            {
                return Err(invalid("zid", &format!("zid {zid} is already taken")));
            }
            zid
        } else if let Some(parent) = params.after.as_deref() {
            Model::find(&txn, parent).await?;
            let siblings: Vec<String> = Entity::find()
                .select_only()
                .column(Column::Zid)
                .filter(Column::Zid.starts_with(parent))
                .into_tuple()
                .all(&txn)
                .await?;
            next_child_id(parent, siblings.iter().map(String::as_str))
        } else {
            let all: Vec<String> = Entity::find()
                .select_only()
                .column(Column::Zid)
                .into_tuple()
                .all(&txn)
                .await?;
            next_root_id(all.iter().map(String::as_str))
        };
        let mut item = Self {
            zid: Set(zid),
            title: Set(rules::text("title", &params.title)?),
            body: Set(params.body),
            kind: Set(params
                .kind
                .as_deref()
                .map_or_else(|| DEFAULT_KIND.to_string(), normalize_kind)),
            source: Set(rules::optional(params.source)),
            resource_id: Set(params.resource_id),
            hippo_id: Set(params.hippo_id),
            ..Default::default()
        };
        sourced(&txn, &mut item).await?;
        // validate up front so a bad kind is a 400, not a database error
        item.validate()?;
        let note = item.insert(&txn).await?;
        note.sync(&txn, params.tags.unwrap_or_default()).await?;
        note.adopt_dangling_links(&txn).await?;
        hippos::weave(&txn, &note).await?;
        txn.commit().await?;
        Ok(note)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// Notes, newest first, narrowed by kind, tag, and a text query on title and body.
    #[must_use]
    pub fn listing(kind: Option<&str>, tag: Option<&str>, q: Option<&str>) -> Select<Self> {
        let mut sel = Self::find();
        if let Some(kind) = kind {
            sel = sel.filter(Column::Kind.eq(kind));
        }
        if let Some(tag) = tag {
            sel = sel.filter(
                Column::Id.in_subquery(
                    Query::select()
                        .column(note_tags::Column::NoteId)
                        .from(note_tags::Entity)
                        .and_where(note_tags::Column::Name.eq(normalize_tag(tag)))
                        .to_owned(),
                ),
            );
        }
        if let Some(q) = q.map(str::trim).filter(|q| !q.is_empty()) {
            let q = q.to_lowercase();
            sel = sel.filter(
                Condition::any()
                    .add(lower(Column::Title).like(contains(&q)))
                    .add(lower(Column::Body).like(contains(&q))),
            );
        }
        sel.order_by_desc(Column::Id)
    }

    /// The inbox: fleeting notes still waiting to be turned into permanent ones.
    #[must_use]
    pub fn inbox() -> Select<Self> {
        Self::listing(Some("fleeting"), None, None)
    }

    /// Notes with no links in or out: ideas not yet connected to anything.
    #[must_use]
    pub fn orphans() -> Select<Self> {
        let ends = |col: note_links::Column| {
            Query::select()
                .column(col)
                .from(note_links::Entity)
                .to_owned()
        };
        Self::find()
            .filter(Column::Id.not_in_subquery(ends(note_links::Column::SourceId)))
            .filter(Column::Id.not_in_subquery(ends(note_links::Column::TargetId)))
            .order_by_desc(Column::Id)
    }

    /// One note picked at random, for serendipitous review.
    pub async fn random<C: ConnectionTrait>(db: &C) -> ModelResult<Option<Model>> {
        Ok(Self::find()
            .order_by(Expr::cust("RANDOM()"), sea_orm::Order::Asc)
            .one(db)
            .await?)
    }

    /// Every note in slipbox order (`1`, `1a`, `1a1`, `1b`, `2`, ..., `10`).
    pub async fn tree<C: ConnectionTrait>(db: &C) -> ModelResult<Vec<Model>> {
        let mut notes = Self::find().all(db).await?;
        notes.sort_by(|a, b| cmp_zid(&a.zid, &b.zid));
        Ok(notes)
    }

    /// Every link as `(source zid, target zid)`.
    pub async fn edges<C: ConnectionTrait>(db: &C) -> ModelResult<Vec<(String, String)>> {
        let zids: HashMap<i64, String> = Self::find()
            .select_only()
            .column(Column::Id)
            .column(Column::Zid)
            .into_tuple::<(i64, String)>()
            .all(db)
            .await?
            .into_iter()
            .collect();
        let links = note_links::Entity::find()
            .order_by_asc(note_links::Column::Id)
            .all(db)
            .await?;
        Ok(links
            .into_iter()
            .filter_map(|l| {
                Some((
                    zids.get(&l.source_id)?.clone(),
                    zids.get(&l.target_id)?.clone(),
                ))
            })
            .collect())
    }

    /// Tags for many notes at once, keyed by note id.
    pub async fn tags_of<C: ConnectionTrait>(
        db: &C,
        ids: &[i64],
    ) -> ModelResult<HashMap<i64, Vec<String>>> {
        let rows = note_tags::Entity::find()
            .filter(note_tags::Column::NoteId.is_in(ids.iter().copied()))
            .order_by_asc(note_tags::Column::Name)
            .all(db)
            .await?;
        let mut map: HashMap<i64, Vec<String>> = HashMap::new();
        for row in rows {
            map.entry(row.note_id).or_default().push(row.name);
        }
        Ok(map)
    }

    /// Counts for the dashboard: notes, links, inbox, orphans, per kind and per tag.
    pub async fn stats<C: ConnectionTrait>(db: &C) -> ModelResult<StatsDto> {
        let by_kind: Vec<(String, i64)> = Self::find()
            .select_only()
            .column(Column::Kind)
            .column_as(Column::Id.count(), "count")
            .group_by(Column::Kind)
            .order_by_asc(Column::Kind)
            .into_tuple()
            .all(db)
            .await?;
        let count = |rows: Vec<(String, i64)>| {
            rows.into_iter()
                .map(|(name, count)| TagCount { name, count })
                .collect()
        };
        Ok(StatsDto {
            notes: Self::find().count(db).await?,
            links: note_links::Entity::find().count(db).await?,
            inbox: Self::inbox().count(db).await?,
            orphans: Self::orphans().count(db).await?,
            by_kind: count(by_kind),
            tags: count(Self::tag_counts(db).await?),
        })
    }

    /// Every tag with how many notes carry it, most used first.
    pub async fn tag_counts<C: ConnectionTrait>(db: &C) -> ModelResult<Vec<(String, i64)>> {
        let mut rows: Vec<(String, i64)> = note_tags::Entity::find()
            .select_only()
            .column(note_tags::Column::Name)
            .column_as(note_tags::Column::Id.count(), "count")
            .group_by(note_tags::Column::Name)
            .into_tuple()
            .all(db)
            .await?;
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        Ok(rows)
    }
}

/// `1`, `21a`, `21a10b`: starts with digits, then alternating letter/digit runs.
#[must_use]
pub fn is_zid(zid: &str) -> bool {
    let segs = zid_segments(zid);
    !segs.is_empty()
        && matches!(segs[0], Seg::Num(_))
        && segs.iter().all(|s| match s {
            Seg::Num(_) => true,
            Seg::Alpha(a) => a.chars().all(|c| c.is_ascii_lowercase()),
        })
        && zid
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
}

#[derive(Debug, PartialEq, Eq)]
enum Seg<'a> {
    Num(u64),
    Alpha(&'a str),
}

fn zid_segments(zid: &str) -> Vec<Seg<'_>> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = zid.as_bytes();
    for i in 1..=bytes.len() {
        if i == bytes.len() || bytes[i].is_ascii_digit() != bytes[start].is_ascii_digit() {
            let part = &zid[start..i];
            out.push(part.parse::<u64>().map_or(Seg::Alpha(part), Seg::Num));
            start = i;
        }
    }
    out
}

/// Slipbox order: numbers compare as numbers, letter runs as `a < b < ... < z < aa`,
/// and a note sorts right before its continuations.
#[must_use]
pub fn cmp_zid(a: &str, b: &str) -> Ordering {
    let (sa, sb) = (zid_segments(a), zid_segments(b));
    for (x, y) in sa.iter().zip(&sb) {
        let ord = match (x, y) {
            (Seg::Num(x), Seg::Num(y)) => x.cmp(y),
            (Seg::Alpha(x), Seg::Alpha(y)) => x.len().cmp(&y.len()).then_with(|| x.cmp(y)),
            (Seg::Num(_), Seg::Alpha(_)) => Ordering::Less,
            (Seg::Alpha(_), Seg::Num(_)) => Ordering::Greater,
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    sa.len().cmp(&sb.len())
}

/// Next continuation under `parent`, given the zids that already exist:
/// `21` → `21a`, then `21b`; `21a` → `21a1`, then `21a2`.
#[must_use]
pub fn next_child_id<'a>(parent: &str, existing: impl IntoIterator<Item = &'a str>) -> String {
    let wants_letter = parent.ends_with(|c: char| c.is_ascii_digit());
    let mut best: Option<&str> = None;
    for id in existing {
        let Some(rest) = id.strip_prefix(parent) else {
            continue;
        };
        let direct_child = !rest.is_empty()
            && if wants_letter {
                rest.chars().all(|c| c.is_ascii_lowercase())
            } else {
                rest.chars().all(|c| c.is_ascii_digit())
            };
        if direct_child && best.is_none_or(|b| (rest.len(), rest) > (b.len(), b)) {
            best = Some(rest);
        }
    }
    let seg = match (wants_letter, best) {
        (true, None) => "a".to_string(),
        (false, None) => "1".to_string(),
        (true, Some(b)) => next_letters(b),
        (false, Some(b)) => (b.parse::<u64>().unwrap_or(0) + 1).to_string(),
    };
    format!("{parent}{seg}")
}

/// Next top-level zid: one past the highest all-digit zid.
#[must_use]
pub fn next_root_id<'a>(existing: impl IntoIterator<Item = &'a str>) -> String {
    let max = existing
        .into_iter()
        .filter_map(|id| id.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    (max + 1).to_string()
}

/// `a` → `b`, `z` → `aa`, `az` → `ba`.
fn next_letters(s: &str) -> String {
    let mut bytes = s.as_bytes().to_vec();
    for i in (0..bytes.len()).rev() {
        if bytes[i] == b'z' {
            bytes[i] = b'a';
        } else {
            bytes[i] += 1;
            return String::from_utf8_lossy(&bytes).into_owned();
        }
    }
    format!("a{}", String::from_utf8_lossy(&bytes))
}

/// The parts of `body` that are prose, as `(text, is_code)` runs: fenced code
/// blocks and inline `code` spans are code, where links and tags do not count.
fn prose_runs(body: &str) -> Vec<(&str, bool)> {
    let mut runs = Vec::new();
    let mut fence: Option<&str> = None;
    let mut start = 0;
    let mut pos = 0;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let marker = ["```", "~~~"].into_iter().find(|m| trimmed.starts_with(m));
        match (fence, marker) {
            (None, Some(m)) => {
                runs.extend(inline_runs(&body[start..pos]));
                start = pos;
                fence = Some(m);
            }
            (Some(open), Some(m)) if open == m => {
                runs.push((&body[start..pos + line.len()], true));
                start = pos + line.len();
                fence = None;
            }
            _ => {}
        }
        pos += line.len();
    }
    if fence.is_some() {
        runs.push((&body[start..], true));
    } else {
        runs.extend(inline_runs(&body[start..]));
    }
    runs
}

/// Split prose at inline `code` spans (a span never crosses a line).
fn inline_runs(text: &str) -> Vec<(&str, bool)> {
    let mut runs = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`').filter(|&c| !after[..c].contains('\n')) else {
            break;
        };
        runs.push((&rest[..open], false));
        runs.push((&rest[open..open + close + 2], true));
        rest = &rest[open + close + 2..];
    }
    runs.push((rest, false));
    runs
}

/// One `[[...]]` split into its parts: `target`, then `#heading`/`^block`
/// (kept with its marker), then the `|label` (kept with its bar).
fn split_wikilink(inner: &str) -> (&str, &str, &str) {
    let (left, label) = inner
        .find('|')
        .map_or((inner, ""), |i| (&inner[..i], &inner[i..]));
    let (target, anchor) = left
        .find(['#', '^'])
        .map_or((left, ""), |i| (&left[..i], &left[i..]));
    (target, anchor, label)
}

/// Every wikilink target in the prose of `body`, in order, with repeats.
fn wikilink_targets(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (text, code) in prose_runs(body) {
        if code {
            continue;
        }
        let mut rest = text;
        while let Some(start) = rest.find("[[") {
            rest = &rest[start + 2..];
            let Some(end) = rest.find("]]") else { break };
            let inner = &rest[..end];
            if !inner.contains('\n') {
                out.push(split_wikilink(inner).0.trim().to_string());
            }
            rest = &rest[end + 2..];
        }
    }
    out
}

/// Targets of `[[target]]`, `[[target|label]]` and Obsidian's `[[target#heading]]`
/// and `[[target^block]]` in `body`, in order, without repeats. Links inside
/// code are not links.
#[must_use]
pub fn wikilinks(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for target in wikilink_targets(body) {
        if !target.is_empty() && !out.contains(&target) {
            out.push(target);
        }
    }
    out
}

/// Rewrite the target of every wikilink in the prose of `body` for which `f`
/// returns a new one, keeping its `#heading`/`^block` and `|label`.
pub fn rewrite_wikilinks(body: &str, f: impl Fn(&str) -> Option<String>) -> String {
    rewrite_wikilinks_with(body, |target, anchor, label| {
        f(target).map(|new| format!("{new}{anchor}{label}"))
    })
}

/// Like [`rewrite_wikilinks`], but `f` sees `(target, "#anchor", "|label")`
/// (empty when absent) and returns the whole new inside of the `[[...]]`.
pub fn rewrite_wikilinks_with(
    body: &str,
    f: impl Fn(&str, &str, &str) -> Option<String>,
) -> String {
    let mut out = String::with_capacity(body.len());
    for (text, code) in prose_runs(body) {
        if code {
            out.push_str(text);
            continue;
        }
        let mut rest = text;
        while let Some(start) = rest.find("[[") {
            let after = &rest[start + 2..];
            let Some(end) = after.find("]]").filter(|&e| !after[..e].contains('\n')) else {
                break;
            };
            let (target, anchor, label) = split_wikilink(&after[..end]);
            out.push_str(&rest[..start]);
            match f(target.trim(), anchor, label) {
                Some(new) => {
                    let _ = write!(out, "[[{new}]]");
                }
                None => out.push_str(&rest[start..start + 2 + end + 2]),
            }
            rest = &after[end + 2..];
        }
        out.push_str(rest);
    }
    out
}

/// `#tags` in the prose of `body` (lowercased). `# Heading`, `a#b`, URL
/// fragments and anything inside code are not tags.
#[must_use]
pub fn hashtags(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (text, code) in prose_runs(body) {
        if code {
            continue;
        }
        let mut prev = ' ';
        for (i, c) in text.char_indices() {
            if c == '#' && (prev.is_whitespace() || prev == '(') {
                let tag: String = text[i + 1..]
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '/'))
                    .collect();
                let tag = tag.trim_end_matches('/');
                if tag.chars().any(char::is_alphabetic) {
                    out.push(normalize_tag(tag));
                }
            }
            prev = c;
        }
    }
    out
}

/// `structure`, `moc` and `index` are what other slip-boxes call a hub.
fn normalize_kind(kind: &str) -> String {
    let kind = kind.trim().to_lowercase();
    match kind.as_str() {
        "structure" | "moc" | "index" => "hub".to_string(),
        _ => kind,
    }
}

/// A literature note says where it comes from: a `source`, or the resource it
/// was written from (whose citation then becomes the source).
async fn sourced<C: ConnectionTrait>(db: &C, item: &mut ActiveModel) -> ModelResult<()> {
    let resource_id = item.resource_id.as_ref().to_owned();
    if let Some(id) = resource_id {
        let resource = resources::Model::find(db, id)
            .await
            .map_err(|_| invalid("resource_id", &format!("resource #{id} does not exist")))?;
        if item.source.as_ref().is_none() {
            item.source = Set(Some(resource.cite()));
        }
    }
    if item.kind.as_ref().as_str() == "literature" && item.source.as_ref().is_none() {
        return Err(invalid(
            "source",
            "a literature note needs a source (book, URL, @citekey) or a resource",
        ));
    }
    Ok(())
}

fn lower(col: Column) -> SimpleExpr {
    Func::lower(Expr::col(col)).into()
}

fn normalize_tag(tag: &str) -> String {
    tag.trim().trim_start_matches('#').to_lowercase()
}

/// `LIKE '%s%'` that matches `s` literally, `%` and `_` included.
fn contains(s: &str) -> LikeExpr {
    LikeExpr::new(format!("%{}%", rules::like_literal(s))).escape('\\')
}

fn yaml_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folgezettel_ids() {
        assert_eq!(next_root_id(["1", "2", "2a", "10"]), "11");
        assert_eq!(next_root_id([]), "1");
        assert_eq!(next_child_id("21", []), "21a");
        assert_eq!(
            next_child_id("21", ["21", "21a", "21b", "21b1", "211"]),
            "21c"
        );
        assert_eq!(next_child_id("21a", ["21a1", "21a9", "21a10"]), "21a11");
        assert_eq!(next_child_id("1", ["1z"]), "1aa");
    }

    #[test]
    fn zid_order_and_shape() {
        let mut zids = vec!["10", "1b", "2", "1a1", "1", "1a", "1aa", "1z"];
        zids.sort_by(|a, b| cmp_zid(a, b));
        assert_eq!(zids, ["1", "1a", "1a1", "1b", "1z", "1aa", "2", "10"]);
        assert!(is_zid("21a3b"));
        assert!(!is_zid("a1"));
        assert!(!is_zid("1A"));
        assert!(!is_zid(""));
    }

    #[test]
    fn parses_wikilinks_and_hashtags() {
        assert_eq!(
            wikilinks("see [[21a]] and [[Zettel|alias]], again [[21a]] [[\n]] [[ ]]"),
            ["21a", "Zettel"]
        );
        assert_eq!(
            hashtags("# Heading\n#rust and (#Learning) a#b #1 #pkm/tools"),
            ["rust", "learning", "pkm/tools"]
        );
    }

    #[test]
    fn code_and_obsidian_anchors() {
        let body = "[[A#h]] [[B^x|b]] `[[C]] #c` #d\n```rust\n[[E]] #e\n```\n[[F]] #f/";
        assert_eq!(wikilinks(body), ["A", "B", "F"]);
        assert_eq!(hashtags(body), ["d", "f"]);
        // an unclosed fence runs to the end
        assert_eq!(wikilinks("[[A]]\n~~~\n[[B]]"), ["A"]);
    }

    #[test]
    fn rewrites_targets_keeping_anchor_and_label() {
        let out = rewrite_wikilinks("[[a]] [[a#h|x]] `[[a]]` [[b]]", |t| {
            (t == "a").then(|| "z".to_string())
        });
        assert_eq!(out, "[[z]] [[z#h|x]] `[[a]]` [[b]]");
    }
}
