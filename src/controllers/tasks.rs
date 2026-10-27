#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    dtos::{
        common::Page,
        tasks::{CreateTask, TaskDto, UpdateTask},
    },
    models::tasks::{ActiveModel, Entity, Model},
};

/// `?project_id=` scopes to one project; `?done=` filters by completion.
#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub project_id: Option<i64>,
    pub done: Option<bool>,
}

#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(pagination): Query<query::PaginationQuery>,
    Query(filter): Query<ListFilter>,
) -> Result<Json<Page<TaskDto>>> {
    let res = query::paginate(
        &ctx.db,
        Entity::listing(filter.project_id, filter.done),
        None,
        &pagination,
    )
    .await?;
    Ok(Json(Page::from_query(res)))
}

#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<TaskDto>> {
    Ok(Json(Model::find(&ctx.db, id).await?.into()))
}

#[debug_handler]
async fn create(State(ctx): State<AppContext>, Json(params): Json<CreateTask>) -> Result<Response> {
    let item = ActiveModel::create(&ctx.db, params).await?;
    Ok((StatusCode::CREATED, Json(TaskDto::from(item))).into_response())
}

#[debug_handler]
async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdateTask>,
) -> Result<Json<TaskDto>> {
    let item = Model::find(&ctx.db, id).await?;
    Ok(Json(item.update_with(&ctx.db, params).await?.into()))
}

#[debug_handler]
async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    Model::find(&ctx.db, id).await?.remove(&ctx.db).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/tasks")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}
