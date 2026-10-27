//! Views over the whole slipbox: what to process, what is unconnected,
//! how it is structured, and a random card to stumble upon.
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;

use crate::{
    dtos::notes::{GraphDto, NoteDetail, NoteDto, StatsDto, TreeEntry},
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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/slipbox")
        .add("/inbox", get(inbox))
        .add("/orphans", get(orphans))
        .add("/random", get(random))
        .add("/tree", get(tree))
        .add("/graph", get(graph))
        .add("/stats", get(stats))
}
