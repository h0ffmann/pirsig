use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;

#[derive(Debug, serde::Serialize, serde::Deserialize, TS)]
pub struct PlastDto {
    pub slug: String,
    /// `math`, `abstraction`, `logic`, `music`, `literature` or `languages`.
    pub domain: String,
    pub title: String,
    pub description: Option<String>,
    /// `ztk` (shipped); `user` is reserved for the person's own plasts.
    pub origin: String,
    /// 1 to 5; rises after three right answers in a row.
    #[ts(type = "number")]
    pub level: i32,
    /// Right answers in a row.
    #[ts(type = "number")]
    pub streak: i32,
    #[ts(type = "string | null")]
    pub due_at: Option<DateTimeWithTimeZone>,
    #[ts(type = "string | null")]
    pub practiced_at: Option<DateTimeWithTimeZone>,
}

impl From<crate::models::_entities::plasts::Model> for PlastDto {
    fn from(m: crate::models::_entities::plasts::Model) -> Self {
        Self {
            slug: m.slug,
            domain: m.domain,
            title: m.title,
            description: m.description,
            origin: m.origin,
            level: m.level,
            streak: m.streak,
            due_at: m.due_at,
            practiced_at: m.practiced_at,
        }
    }
}

/// An exercise to show. Answer it with `plast`, `seed` and `level` as given.
#[derive(Debug, serde::Serialize, serde::Deserialize, TS)]
pub struct ExerciseDto {
    pub plast: String,
    #[ts(type = "number")]
    pub seed: u64,
    #[ts(type = "number")]
    pub level: u8,
    pub prompt: String,
    /// Multiple choice; answer with the letter (A–D) or the text.
    pub choices: Vec<String>,
}

impl From<crate::plasticity::Exercise> for ExerciseDto {
    fn from(e: crate::plasticity::Exercise) -> Self {
        Self {
            plast: e.plast.to_string(),
            seed: e.seed,
            level: e.level,
            prompt: e.prompt,
            choices: e.choices,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct AnswerExercise {
    pub plast: String,
    #[ts(type = "number")]
    pub seed: u64,
    #[ts(type = "number")]
    pub level: u8,
    pub answer: String,
    /// How long the answer took, for the person's own record.
    #[ts(type = "number | null")]
    pub millis: Option<i64>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, TS)]
pub struct AnswerDto {
    pub correct: bool,
    pub expected: String,
    /// Why: the rule, the form, the author.
    pub explain: String,
    /// The plast after this answer: its new level, streak and next due time.
    pub plast: PlastDto,
}
