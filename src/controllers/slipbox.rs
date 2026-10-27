//! Views over the whole slipbox: what to process, what is unconnected,
//! how it is structured, and a random card to stumble upon.
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;

use axum::http::header;
use serde::Deserialize;

use crate::{
    dtos::notes::{GraphDto, NoteDetail, NoteDto, StatsDto, TreeEntry},
    ingest::rss,
    models::notes::Entity,
};

#[debug_handler]
async fn inbox(State(ctx): State<AppContext>) -> Result<Json<Vec<NoteDto>>> {
    let notes = Entity::inbox().all(&ctx.db).await?;
    Ok(Json(NoteDto::many(&ctx.db, notes).await?))
}

#[debug_handler]
async fn orphans(State(ctx): State<AppContext>) -> Result<Json<Vec<NoteDto>>> {
    let notes = Entity::orphans().all(&ctx.db).await?;
    Ok(Json(NoteDto::many(&ctx.db, notes).await?))
}

#[debug_handler]
async fn random(State(ctx): State<AppContext>) -> Result<Json<NoteDetail>> {
    let note = Entity::random(&ctx.db).await?.ok_or(Error::NotFound)?;
    Ok(Json(NoteDetail::load(&ctx.db, note).await?))
}

#[debug_handler]
async fn tree(State(ctx): State<AppContext>) -> Result<Json<Vec<TreeEntry>>> {
    let notes = Entity::tree(&ctx.db).await?;
    Ok(Json(notes.iter().map(TreeEntry::from).collect()))
}

#[debug_handler]
async fn graph(State(ctx): State<AppContext>) -> Result<Json<GraphDto>> {
    Ok(Json(GraphDto::load(&ctx.db).await?))
}

#[debug_handler]
async fn stats(State(ctx): State<AppContext>) -> Result<Json<StatsDto>> {
    Ok(Json(Entity::stats(&ctx.db).await?))
}

/// `?kind=permanent&tag=pkm&limit=50`.
#[derive(Debug, Deserialize)]
pub struct FeedParams {
    pub kind: Option<String>,
    pub tag: Option<String>,
    pub limit: Option<u64>,
}

/// The slipbox out (`ztk.web.rss`): permanent notes by default, newest first.
#[debug_handler]
async fn feed(State(ctx): State<AppContext>, Query(params): Query<FeedParams>) -> Result<Response> {
    let limit = params.limit.unwrap_or(50).clamp(1, 500);
    let items = rss::feed_notes(
        &ctx.db,
        params.kind.as_deref(),
        params.tag.as_deref(),
        limit,
    )
    .await?;
    let xml = rss::render("ztk slipbox", &ctx.config.server.full_url(), &items);
    Ok((
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml,
    )
        .into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/slipbox")
        .add("/inbox", get(inbox))
        .add("/orphans", get(orphans))
        .add("/random", get(random))
        .add("/tree", get(tree))
        .add("/graph", get(graph))
        .add("/stats", get(stats))
        .add("/feed.xml", get(feed))
}
