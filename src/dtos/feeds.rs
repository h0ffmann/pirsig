use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct FeedDto {
    #[ts(type = "number")]
    pub id: i64,
    pub url: String,
    /// The feed's own title, learnt on the first pull.
    pub title: Option<String>,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
    #[ts(type = "string | null")]
    pub last_fetched_at: Option<DateTimeWithTimeZone>,
}

impl From<crate::models::_entities::feeds::Model> for FeedDto {
    fn from(f: crate::models::_entities::feeds::Model) -> Self {
        Self {
            id: f.id,
            url: f.url,
            title: f.title,
            area_id: f.area_id,
            project_id: f.project_id,
            last_fetched_at: f.last_fetched_at,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateFeed {
    pub url: String,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct IngestArxiv {
    /// Ids in any form: `1706.03762`, `arXiv:1706.03762v5`, abs/ or pdf/ URLs.
    pub ids: Vec<String>,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
    /// Also put a fleeting "Read:" note in the inbox.
    #[serde(default)]
    pub note: bool,
}
