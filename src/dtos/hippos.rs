use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

use crate::dtos::common::Patch;

#[derive(Debug, serde::Serialize, serde::Deserialize, TS)]
pub struct HippoDto {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    /// Obsessions and special interests: the plateaus the rhizome grows around.
    pub fixations: Vec<String>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::hippos::Model> for HippoDto {
    fn from(m: crate::models::_entities::hippos::Model) -> Self {
        Self {
            id: m.id,
            fixations: crate::models::hippos::split(&m.fixations),
            name: m.name,
            description: m.description,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateHippo {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub fixations: Vec<String>,
}

/// Partial update: `fixations`, when present, replaces the list.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateHippo {
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub description: Patch<String>,
    pub fixations: Option<Vec<String>>,
}

/// A note as the map shows it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TS)]
pub struct NodeRef {
    pub zid: String,
    pub title: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, TS)]
pub struct Plateau {
    pub fixation: String,
    pub notes: Vec<NodeRef>,
}

/// The rhizome of one Hippo: many entry points, no root.
#[derive(Debug, serde::Serialize, serde::Deserialize, TS)]
pub struct RhizomeMap {
    pub hippo: HippoDto,
    #[ts(type = "number")]
    pub notes: usize,
    /// Links between two notes of this Hippo.
    #[ts(type = "number")]
    pub links: usize,
    /// Where you left off: the notes touched last.
    pub resume: Vec<NodeRef>,
    /// Notes around each fixation.
    pub plateaus: Vec<Plateau>,
    /// Connected pieces, largest first. One piece is a whole rhizome.
    pub components: Vec<Vec<NodeRef>>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct WeaveReport {
    /// Links made for notes that had too few.
    #[ts(type = "number")]
    pub woven: usize,
    /// Links made to join separate pieces.
    #[ts(type = "number")]
    pub bridged: usize,
}
