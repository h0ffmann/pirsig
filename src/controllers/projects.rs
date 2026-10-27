#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    dtos::{
        common::Page,
        projects::{CreateProject, ProjectDto, UpdateProject},
    },
    models::projects::{ActiveModel, Entity, Model},
};

/// `?area_id=` scopes to one area; `?archived=true` lists the archive.
#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub area_id: Option<i64>,
    #[serde(default)]
    pub archived: bool,
}

#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(pagination): Query<query::PaginationQuery>,
    Query(filter): Query<ListFilter>,
) -> Result<Json<Page<ProjectDto>>> {
    let res = query::paginate(
        &ctx.db,
        Entity::listing(filter.area_id, filter.archived),
        None,
        &pagination,
    )
    .await?;
    Ok(Json(Page::from_query(res)))
}

#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<ProjectDto>> {
    Ok(Json(Model::find(&ctx.db, id).await?.into()))
}

#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateProject>,
) -> Result<Response> {
    let item = ActiveModel::create(&ctx.db, params).await?;
    Ok((StatusCode::CREATED, Json(ProjectDto::from(item))).into_response())
}

#[debug_handler]
async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdateProject>,
) -> Result<Json<ProjectDto>> {
    let item = Model::find(&ctx.db, id).await?;
    Ok(Json(item.update_with(&ctx.db, params).await?.into()))
}

#[debug_handler]
async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    Model::find(&ctx.db, id).await?.remove(&ctx.db).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[debug_handler]
async fn archive(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<ProjectDto>> {
    let item = Model::find(&ctx.db, id).await?;
    Ok(Json(item.set_archived(&ctx.db, true).await?.into()))
}

#[debug_handler]
async fn unarchive(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<ProjectDto>> {
    let item = Model::find(&ctx.db, id).await?;
    Ok(Json(item.set_archived(&ctx.db, false).await?.into()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/projects")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
        .add("/{id}/archive", post(archive))
        .add("/{id}/unarchive", post(unarchive))
}
