use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

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
}

/// Partial update: only the fields that are present are changed.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateResource {
    pub title: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
}
