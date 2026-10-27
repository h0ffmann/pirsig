use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

use crate::dtos::common::Patch;

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct AreaDto {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    #[ts(type = "string | null")]
    pub archived_at: Option<DateTimeWithTimeZone>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::areas::Model> for AreaDto {
    fn from(m: crate::models::_entities::areas::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            archived_at: m.archived_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateArea {
    pub name: String,
    pub description: Option<String>,
}

/// Partial update: only the fields that are present are changed; `null` clears
/// an optional field.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateArea {
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub description: Patch<String>,
}
