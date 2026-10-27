//! Hippo identities and their rhizomes (`/api/hippos`).
#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;

use crate::{
    dtos::hippos::{CreateHippo, HippoDto, RhizomeMap, UpdateHippo, WeaveReport},
    models::hippos::{ActiveModel, Entity, Model},
};

#[debug_handler]
async fn list(State(ctx): State<AppContext>) -> Result<Json<Vec<HippoDto>>> {
    let all = Entity::listing().all(&ctx.db).await?;
    Ok(Json(all.into_iter().map(Into::into).collect()))
}

#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<HippoDto>> {
    Ok(Json(Model::find(&ctx.db, id).await?.into()))
}

#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateHippo>,
) -> Result<Response> {
    let h = ActiveModel::create(&ctx.db, params).await?;
    Ok((StatusCode::CREATED, Json(HippoDto::from(h))).into_response())
}

#[debug_handler]
async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdateHippo>,
) -> Result<Json<HippoDto>> {
    let h = Model::find(&ctx.db, id).await?;
    Ok(Json(h.update_with(&ctx.db, params).await?.into()))
}

#[debug_handler]
async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    Model::find(&ctx.db, id).await?.remove(&ctx.db).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[debug_handler]
async fn map(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<RhizomeMap>> {
    Ok(Json(Model::find(&ctx.db, id).await?.map(&ctx.db).await?))
}

#[debug_handler]
async fn weave(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<WeaveReport>> {
    Ok(Json(
        Model::find(&ctx.db, id).await?.weave_all(&ctx.db).await?,
    ))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/hippos")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
        .add("/{id}/map", get(map))
        .add("/{id}/weave", post(weave))
}
