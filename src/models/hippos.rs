//! Hippo identities and their rhizomes.
//!
//! A Hippo is a self-described way of thinking: ADHD, autistic, neurotic,
//! obsessive, or any mix. It is never a diagnosis, and it stays in the private
//! brain. Its notes do not form a Folgezettel tree. They form a rhizome
//! (Deleuze and Guattari, *A Thousand Plateaus*, 1980): any note can connect to
//! any other, there is no trunk, and every point is an entry point. Pirsig keeps
//! that true without asking for filing work:
//!
//! - **Connection.** Each note is woven in on save: linked to the notes it
//!   shares fixations, tags or title words with, else to the last thing
//!   captured, until it has [`RHIZOME_MIN`] links. Every link says why.
//! - **No trunk.** A Hippo note is never filed under another (`after`).
//! - **Rupture heals.** Deleting a note links its neighbours to each other,
//!   so the network never falls into pieces.
//! - **Plateaus.** Fixations (obsessions, special interests) are where notes
//!   gather; the map shows each one, and where you left off.
#![allow(clippy::missing_errors_doc)]
use std::collections::{HashMap, HashSet};

pub use super::_entities::hippos::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{
    entity::prelude::*, ActiveValue::Set, Condition, IntoActiveModel, QueryOrder,
    TransactionSession, TransactionTrait,
};

use super::{note_links, notes, rules};
use crate::dtos::hippos::{
    CreateHippo, HippoDto, NodeRef, Plateau, RhizomeMap, UpdateHippo, WeaveReport,
};
pub type Hippos = Entity;

/// Self-described traits. Pirsig adapts nothing clinical to them; the `hippo`
/// skill uses them to adapt how Claude helps (short steps, literal wording,
/// reassurance, room for deep dives).
pub const TRAITS: [&str; 4] = ["adhd", "autistic", "neurotic", "obsessive"];

/// A note is woven in until it has this many links (in or out).
pub const RHIZOME_MIN: usize = 2;

/// Prefix of the reason on every link the rhizome makes, so they can be told
/// from the person's own.
pub const REASON: &str = "rhizome:";

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

/// A stored comma list as a list.
#[must_use]
pub fn split(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect()
}

fn traits(values: Vec<String>) -> ModelResult<String> {
    let mut out: Vec<String> = Vec::new();
    for t in values {
        let t = t.trim().to_lowercase();
        if t.is_empty() {
            continue;
        }
        rules::one_of("traits", &t, &TRAITS)?;
        if !out.contains(&t) {
            out.push(t);
        }
    }
    Ok(out.join(","))
}

fn fixations(values: &[String]) -> String {
    let mut out: Vec<String> = Vec::new();
    for f in values.iter().flat_map(|v| v.split(',')) {
        let f = f.trim().trim_start_matches('#').to_lowercase();
        if !f.is_empty() && !out.contains(&f) {
            out.push(f);
        }
    }
    out.join(",")
}

impl Model {
    /// Load a Hippo or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    pub async fn update_with<C: ConnectionTrait>(
        self,
        db: &C,
        params: UpdateHippo,
    ) -> ModelResult<Self> {
        let mut item = self.into_active_model();
        if let Some(name) = params.name {
            item.name = Set(rules::text("name", &name)?);
        }
        if let Some(description) = params.description.into_option() {
            item.description = Set(rules::optional(description));
        }
        if let Some(t) = params.traits {
            item.traits = Set(traits(t)?);
        }
        if let Some(f) = params.fixations {
            item.fixations = Set(fixations(&f));
        }
        Ok(item.update(db).await?)
    }

    /// Delete the identity. Its notes stay in the slipbox, with their links.
    pub async fn remove<C: ConnectionTrait + TransactionTrait>(self, db: &C) -> ModelResult<()> {
        let txn = db.begin().await?;
        notes::Entity::update_many()
            .col_expr(notes::Column::HippoId, Expr::value(Option::<i64>::None))
            .filter(notes::Column::HippoId.eq(self.id))
            .exec(&txn)
            .await?;
        self.into_active_model().delete(&txn).await?;
        txn.commit().await?;
        Ok(())
    }

    async fn notes<C: ConnectionTrait>(&self, db: &C) -> ModelResult<Vec<notes::Model>> {
        Ok(notes::Entity::find()
            .filter(notes::Column::HippoId.eq(self.id))
            .order_by_asc(notes::Column::Id)
            .all(db)
            .await?)
    }

    /// The rhizome as a map: where you left off, the plateaus, the pieces.
    pub async fn map<C: ConnectionTrait>(&self, db: &C) -> ModelResult<RhizomeMap> {
        let net = Net::load(db, self).await?;
        let mut recent: Vec<&notes::Model> = net.notes.iter().collect();
        recent.sort_by_key(|n| std::cmp::Reverse((n.updated_at, n.id)));
        let plateaus = split(&self.fixations)
            .into_iter()
            .map(|f| Plateau {
                notes: net
                    .notes
                    .iter()
                    .filter(|n| net.about(n).contains(&f))
                    .map(node)
                    .collect(),
                fixation: f,
            })
            .collect();
        let by_id: HashMap<i64, &notes::Model> = net.notes.iter().map(|n| (n.id, n)).collect();
        Ok(RhizomeMap {
            notes: net.notes.len(),
            links: net.edges.len(),
            resume: recent.into_iter().take(5).map(node).collect(),
            plateaus,
            components: net
                .components()
                .into_iter()
                .map(|c| c.iter().map(|id| node(by_id[id])).collect())
                .collect(),
            hippo: HippoDto::from(self.clone()),
        })
    }

    /// Weave every note that has too few links, then bridge the pieces until
    /// the rhizome is one.
    pub async fn weave_all<C: ConnectionTrait + TransactionTrait>(
        &self,
        db: &C,
    ) -> ModelResult<WeaveReport> {
        let txn = db.begin().await?;
        let mut report = WeaveReport::default();
        for note in self.notes(&txn).await? {
            report.woven += weave(&txn, &note).await?;
        }
        let net = Net::load(&txn, self).await?;
        let pieces = net.components();
        if let Some((main, rest)) = pieces.split_first() {
            for piece in rest {
                let from = net.newest(piece);
                let (to, why) = net.best_match(from, main);
                from.link_to(&txn, to, Some(why)).await?;
                report.bridged += 1;
            }
        }
        txn.commit().await?;
        Ok(report)
    }
}

impl ActiveModel {
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateHippo) -> ModelResult<Model> {
        Ok(Self {
            name: Set(rules::text("name", &params.name)?),
            description: Set(rules::optional(params.description)),
            traits: Set(traits(params.traits)?),
            fixations: Set(fixations(&params.fixations)),
            ..Default::default()
        }
        .insert(db)
        .await?)
    }
}

impl Entity {
    #[must_use]
    pub fn listing() -> Select<Self> {
        Self::find().order_by_asc(Column::Id)
    }
}

fn node(n: &notes::Model) -> NodeRef {
    NodeRef {
        zid: n.zid.clone(),
        title: n.title.clone(),
    }
}

/// Words that say nothing about what a note is about.
const STOP: [&str; 24] = [
    "the", "and", "for", "with", "that", "this", "from", "into", "about", "what", "when", "why",
    "how", "are", "was", "but", "not", "you", "your", "its", "it's", "have", "has", "uma",
];

/// A Hippo's notes and the links among them, for the rhizome rules.
struct Net {
    notes: Vec<notes::Model>,
    edges: Vec<(i64, i64)>,
    tags: HashMap<i64, Vec<String>>,
    fixations: Vec<String>,
}

impl Net {
    async fn load<C: ConnectionTrait>(db: &C, hippo: &Model) -> ModelResult<Self> {
        let notes = hippo.notes(db).await?;
        let ids: Vec<i64> = notes.iter().map(|n| n.id).collect();
        let edges = note_links::Entity::find()
            .filter(note_links::Column::SourceId.is_in(ids.clone()))
            .filter(note_links::Column::TargetId.is_in(ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .map(|l| (l.source_id, l.target_id))
            .collect();
        Ok(Self {
            tags: notes::Entity::tags_of(db, &ids).await?,
            notes,
            edges,
            fixations: split(&hippo.fixations),
        })
    }

    /// The fixations a note is about: tagged with one, or naming it.
    fn about(&self, n: &notes::Model) -> Vec<String> {
        let tags = self.tags.get(&n.id).map_or(&[][..], Vec::as_slice);
        let text = format!("{} {}", n.title, n.body).to_lowercase();
        self.fixations
            .iter()
            .filter(|f| {
                let as_tag = f.replace(' ', "-");
                tags.iter()
                    .any(|t| *t == as_tag || t.starts_with(&format!("{as_tag}/")))
                    || text.contains(f.as_str())
            })
            .cloned()
            .collect()
    }

    fn words(n: &notes::Model) -> HashSet<String> {
        n.title
            .split(|c: char| !c.is_alphanumeric())
            .map(str::to_lowercase)
            .filter(|w| w.chars().count() > 3 && !STOP.contains(&w.as_str()))
            .collect()
    }

    /// How much two notes share, and the reason to give for linking them.
    fn affinity(&self, a: &notes::Model, b: &notes::Model) -> (usize, String) {
        let (fa, fb) = (self.about(a), self.about(b));
        if let Some(f) = fa.iter().find(|f| fb.contains(f)) {
            return (3, format!("{REASON} both circle {f}"));
        }
        let ta = self.tags.get(&a.id).map_or(&[][..], Vec::as_slice);
        let tb = self.tags.get(&b.id).map_or(&[][..], Vec::as_slice);
        if let Some(t) = ta.iter().find(|t| tb.contains(t)) {
            return (2, format!("{REASON} both carry #{t}"));
        }
        let wa = Self::words(a);
        if let Some(w) = Self::words(b).iter().find(|w| wa.contains(*w)) {
            return (1, format!("{REASON} both speak of \"{w}\""));
        }
        (0, String::new())
    }

    fn linked(&self, a: i64, b: i64) -> bool {
        self.edges
            .iter()
            .any(|&(s, t)| (s == a && t == b) || (s == b && t == a))
    }

    fn degree(&self, id: i64) -> usize {
        self.edges
            .iter()
            .filter(|&&(s, t)| s == id || t == id)
            .count()
    }

    /// Connected pieces (links read both ways), largest first.
    fn components(&self) -> Vec<Vec<i64>> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for n in &self.notes {
            if !seen.insert(n.id) {
                continue;
            }
            let mut piece = vec![n.id];
            let mut i = 0;
            while i < piece.len() {
                let at = piece[i];
                for &(s, t) in &self.edges {
                    let other = if s == at {
                        t
                    } else if t == at {
                        s
                    } else {
                        continue;
                    };
                    if seen.insert(other) {
                        piece.push(other);
                    }
                }
                i += 1;
            }
            piece.sort_unstable();
            out.push(piece);
        }
        out.sort_by_key(|p| std::cmp::Reverse(p.len()));
        out
    }

    fn get(&self, id: i64) -> &notes::Model {
        self.notes
            .iter()
            .find(|n| n.id == id)
            .unwrap_or(&self.notes[0])
    }

    fn newest(&self, ids: &[i64]) -> &notes::Model {
        ids.iter()
            .map(|id| self.get(*id))
            .max_by_key(|n| (n.updated_at, n.id))
            .unwrap_or(&self.notes[0])
    }

    /// The note in `among` that `from` shares most with, else the newest.
    fn best_match(&self, from: &notes::Model, among: &[i64]) -> (&notes::Model, String) {
        among
            .iter()
            .map(|id| self.get(*id))
            .map(|n| (n, self.affinity(from, n)))
            .filter(|(_, (score, _))| *score > 0)
            .max_by_key(|(n, (score, _))| (*score, n.updated_at, n.id))
            .map_or_else(
                || {
                    let to = self.newest(among);
                    (to, format!("{REASON} bridges to \"{}\"", to.title))
                },
                |(n, (_, why))| (n, why),
            )
    }
}

/// Weave `note` into its Hippo's rhizome until it has [`RHIZOME_MIN`] links;
/// returns how many links were made. A note outside any Hippo is left alone.
pub async fn weave<C: ConnectionTrait>(db: &C, note: &notes::Model) -> ModelResult<usize> {
    let Some(hippo_id) = note.hippo_id else {
        return Ok(0);
    };
    let hippo = Model::find(db, hippo_id).await?;
    let net = Net::load(db, &hippo).await?;
    let mut need = RHIZOME_MIN.saturating_sub(net.degree(note.id));
    if need == 0 {
        return Ok(0);
    }
    let mut candidates: Vec<(&notes::Model, usize, String)> = net
        .notes
        .iter()
        .filter(|n| n.id != note.id && !net.linked(n.id, note.id))
        .map(|n| {
            let (score, why) = net.affinity(note, n);
            (n, score, why)
        })
        .filter(|(_, score, _)| *score > 0)
        .collect();
    candidates.sort_by_key(|(n, score, _)| std::cmp::Reverse((*score, n.updated_at, n.id)));
    let mut made = 0;
    for (to, _, why) in candidates.into_iter().take(need) {
        note.link_to(db, to, Some(why)).await?;
        made += 1;
    }
    need -= made;
    // nothing in common yet: join the stream of thought where it was last
    if need > 0 && net.degree(note.id) + made == 0 {
        let last = net
            .notes
            .iter()
            .filter(|n| n.id != note.id)
            .max_by_key(|n| (n.updated_at, n.id));
        if let Some(last) = last {
            let why = format!("{REASON} captured next to \"{}\"", last.title);
            note.link_to(db, last, Some(why)).await?;
            made += 1;
        }
    }
    Ok(made)
}

/// Before `note` goes, link its neighbours to each other in a chain, so the
/// rupture does not split the rhizome.
pub async fn heal<C: ConnectionTrait>(db: &C, note: &notes::Model) -> ModelResult<usize> {
    if note.hippo_id.is_none() {
        return Ok(0);
    }
    let links = note_links::Entity::find()
        .filter(
            Condition::any()
                .add(note_links::Column::SourceId.eq(note.id))
                .add(note_links::Column::TargetId.eq(note.id)),
        )
        .all(db)
        .await?;
    let mut ids: Vec<i64> = links
        .iter()
        .map(|l| {
            if l.source_id == note.id {
                l.target_id
            } else {
                l.source_id
            }
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let neighbours = notes::Entity::find()
        .filter(notes::Column::Id.is_in(ids))
        .filter(notes::Column::HippoId.eq(note.hippo_id))
        .order_by_asc(notes::Column::Id)
        .all(db)
        .await?;
    let mut made = 0;
    for pair in neighbours.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let exists = note_links::Entity::find()
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(note_links::Column::SourceId.eq(a.id))
                            .add(note_links::Column::TargetId.eq(b.id)),
                    )
                    .add(
                        Condition::all()
                            .add(note_links::Column::SourceId.eq(b.id))
                            .add(note_links::Column::TargetId.eq(a.id)),
                    ),
            )
            .one(db)
            .await?
            .is_some();
        if !exists {
            let why = format!("{REASON} reconnected through \"{}\"", note.title);
            a.link_to(db, b, Some(why)).await?;
            made += 1;
        }
    }
    Ok(made)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traits_and_fixations_are_normalized() {
        assert_eq!(
            traits(vec!["ADHD".into(), " adhd".into(), "autistic".into()]).unwrap(),
            "adhd,autistic"
        );
        assert!(traits(vec!["bipolar-ish".into()]).is_err());
        assert_eq!(
            fixations(&["#Rogue Waves".into(), "chess, openings".into()]),
            "rogue waves,chess,openings"
        );
        assert_eq!(split("a, b,,c"), ["a", "b", "c"]);
    }
}
