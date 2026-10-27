use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct ProjectDto {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    #[ts(type = "number")]
    pub area_id: i64,
    #[ts(type = "string | null")]
    pub archived_at: Option<DateTimeWithTimeZone>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::projects::Model> for ProjectDto {
    fn from(m: crate::models::_entities::projects::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            status: m.status,
            area_id: m.area_id,
            archived_at: m.archived_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    #[ts(type = "number")]
    pub area_id: i64,
}

/// Partial update: only the fields that are present are changed.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
}
