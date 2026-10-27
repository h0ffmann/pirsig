//! Plasticity: quick exercises (`/api/plasticity`).
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    dtos::plasticity::{AnswerDto, AnswerExercise, ExerciseDto, PlastDto},
    models::plasts::{self, Entity, Model},
};

/// `?domain=music`.
#[derive(Debug, Deserialize)]
pub struct DomainFilter {
    pub domain: Option<String>,
}

#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(f): Query<DomainFilter>,
) -> Result<Json<Vec<PlastDto>>> {
    let all = Entity::listing(f.domain.as_deref(), &ctx.db).await?;
    Ok(Json(all.into_iter().map(Into::into).collect()))
}

/// The next exercise: the most due plast, at its level, from a fresh seed.
#[debug_handler]
async fn next(
    State(ctx): State<AppContext>,
    Query(f): Query<DomainFilter>,
) -> Result<Json<ExerciseDto>> {
    let plast = Entity::next(&ctx.db, f.domain.as_deref()).await?;
    Ok(Json(plast.exercise(plasts::fresh_seed(), None)?.into()))
}

#[debug_handler]
async fn answer(
    State(ctx): State<AppContext>,
    Json(params): Json<AnswerExercise>,
) -> Result<Json<AnswerDto>> {
    let plast = Model::find(&ctx.db, &params.plast).await?;
    let (plast, ex, correct) = plast
        .answer(
            &ctx.db,
            params.seed,
            params.level,
            &params.answer,
            params.millis,
        )
        .await?;
    Ok(Json(AnswerDto {
        correct,
        expected: ex.expected(),
        explain: ex.explain,
        plast: plast.into(),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/plasticity")
        .add("/", get(list))
        .add("/next", get(next))
        .add("/answer", post(answer))
}
