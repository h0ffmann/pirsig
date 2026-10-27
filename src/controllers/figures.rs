#![allow(clippy::unused_async)]
use loco_rs::prelude::*;

use crate::figures::{archify, graphify, Brain, Scope};

/// `GET /api/figures/tree?archived=true&done=true`: the PARA tree as archify JSON IR.
#[debug_handler]
async fn tree(State(ctx): State<AppContext>, Query(scope): Query<Scope>) -> Result<Response> {
    let brain = Brain::load(&ctx.db, scope).await?;
    format::json(archify::tree(&brain, &archify::Options::default()))
}

/// `GET /api/figures/graph?archived=true&done=true`: the brain as a graphify graph.json.
#[debug_handler]
async fn graph(State(ctx): State<AppContext>, Query(scope): Query<Scope>) -> Result<Response> {
    let brain = Brain::load(&ctx.db, scope).await?;
    format::json(graphify::graph(&brain))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/figures")
        .add("/tree", get(tree))
        .add("/graph", get(graph))
}
