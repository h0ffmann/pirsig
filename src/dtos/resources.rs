use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

use crate::dtos::common::Patch;

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct ResourceDto {
    #[ts(type = "number")]
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub body: Option<String>,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
    /// `article`, `book`, `paper`, `repo`, `video`, `web` or `other`.
    pub kind: Option<String>,
    /// BibTeX key (`vaswani2017attention`): how literature notes cite it.
    pub citekey: Option<String>,
    /// Where an ingest found it (`arxiv:1706.03762`, `rss:<guid>`); re-ingesting updates it in place.
    pub external_id: Option<String>,
    #[ts(type = "string | null")]
    pub archived_at: Option<DateTimeWithTimeZone>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::resources::Model> for ResourceDto {
    fn from(m: crate::models::_entities::resources::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            url: m.url,
            body: m.body,
            area_id: m.area_id,
            project_id: m.project_id,
            kind: m.kind,
            citekey: m.citekey,
            external_id: m.external_id,
            archived_at: m.archived_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateResource {
    pub title: String,
    pub url: Option<String>,
    pub body: Option<String>,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
    pub kind: Option<String>,
    pub citekey: Option<String>,
    pub external_id: Option<String>,
}

/// Partial update: only the fields that are present are changed; `null` clears
/// an optional field.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateResource {
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub url: Patch<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub body: Patch<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "number | null")]
    pub area_id: Patch<i64>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "number | null")]
    pub project_id: Patch<i64>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub kind: Patch<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub citekey: Patch<String>,
}
