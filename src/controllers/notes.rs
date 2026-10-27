#![allow(clippy::unused_async)]
use axum::http::{header, StatusCode};
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    dtos::{
        common::Page,
        notes::{CreateLink, CreateNote, NoteDetail, NoteDto, UpdateNote},
    },
    models::notes::{ActiveModel, Entity, Model},
};

/// `?q=rust&kind=permanent&tag=pkm` narrows the listing.
#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub q: Option<String>,
    pub kind: Option<String>,
    pub tag: Option<String>,
}

#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(pagination): Query<query::PaginationQuery>,
    Query(filter): Query<ListFilter>,
) -> Result<Json<Page<NoteDto>>> {
    let sel = Entity::listing(
        filter.kind.as_deref(),
        filter.tag.as_deref(),
        filter.q.as_deref(),
    );
    let res = query::paginate(&ctx.db, sel, None, &pagination).await?;
    let items = NoteDto::many(&ctx.db, res.page).await?;
    Ok(Json(Page {
        items,
        page: res.meta.page,
        page_size: res.meta.page_size,
        total_pages: res.meta.total_pages,
        total_items: res.meta.total_items,
    }))
}

#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path(zid): Path<String>,
) -> Result<Json<NoteDetail>> {
    let note = Model::find(&ctx.db, &zid).await?;
    Ok(Json(NoteDetail::load(&ctx.db, note).await?))
}

#[debug_handler]
async fn markdown(State(ctx): State<AppContext>, Path(zid): Path<String>) -> Result<Response> {
    let note = Model::find(&ctx.db, &zid).await?;
    let md = note.to_markdown(&note.tags(&ctx.db).await?, &note.links(&ctx.db).await?);
    Ok(([(header::CONTENT_TYPE, "text/markdown; charset=utf-8")], md).into_response())
}

#[debug_handler]
async fn create(State(ctx): State<AppContext>, Json(params): Json<CreateNote>) -> Result<Response> {
    let note = ActiveModel::create(&ctx.db, params).await?;
    Ok((
        StatusCode::CREATED,
        Json(NoteDetail::load(&ctx.db, note).await?),
    )
        .into_response())
}

#[debug_handler]
async fn update(
    State(ctx): State<AppContext>,
    Path(zid): Path<String>,
    Json(params): Json<UpdateNote>,
) -> Result<Json<NoteDetail>> {
    let note = Model::find(&ctx.db, &zid).await?;
    let note = note.update_with(&ctx.db, params).await?;
    Ok(Json(NoteDetail::load(&ctx.db, note).await?))
}

#[debug_handler]
async fn remove(State(ctx): State<AppContext>, Path(zid): Path<String>) -> Result<Response> {
    Model::find(&ctx.db, &zid).await?.remove(&ctx.db).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[debug_handler]
async fn link(
    State(ctx): State<AppContext>,
    Path(zid): Path<String>,
    Json(params): Json<CreateLink>,
) -> Result<Json<NoteDetail>> {
    let note = Model::find(&ctx.db, &zid).await?;
    let target = Model::find(&ctx.db, &params.target).await?;
    note.link_to(&ctx.db, &target, params.context).await?;
    Ok(Json(NoteDetail::load(&ctx.db, note).await?))
}

#[debug_handler]
async fn unlink(
    State(ctx): State<AppContext>,
    Path((zid, target)): Path<(String, String)>,
) -> Result<Response> {
    let note = Model::find(&ctx.db, &zid).await?;
    let target = Model::find(&ctx.db, &target).await?;
    if note.unlink(&ctx.db, &target).await? {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(Error::NotFound)
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/notes")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{zid}", get(get_one))
        .add("/{zid}", put(update))
        .add("/{zid}", patch(update))
        .add("/{zid}", delete(remove))
        .add("/{zid}/markdown", get(markdown))
        .add("/{zid}/links", post(link))
        .add("/{zid}/links/{target}", delete(unlink))
}
