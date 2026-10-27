use sea_orm::prelude::{Date, DateTimeWithTimeZone};
use ts_rs::TS;

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct TaskDto {
    #[ts(type = "number")]
    pub id: i64,
    pub title: String,
    pub notes: Option<String>,
    pub done: bool,
    #[ts(type = "string | null")]
    pub due_on: Option<Date>,
    #[ts(type = "number")]
    pub project_id: i64,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::tasks::Model> for TaskDto {
    fn from(m: crate::models::_entities::tasks::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            notes: m.notes,
            done: m.done,
            due_on: m.due_on,
            project_id: m.project_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateTask {
    pub title: String,
    pub notes: Option<String>,
    #[serde(default)]
    pub done: bool,
    #[ts(type = "string | null")]
    pub due_on: Option<Date>,
    #[ts(type = "number")]
    pub project_id: i64,
}

/// Partial update: only the fields that are present are changed.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub done: Option<bool>,
    #[ts(type = "string | null")]
    pub due_on: Option<Date>,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
}
