//! RSS/Atom subscriptions (`ztk.web.rss` in) and arXiv ingest (`ztk.sci.arxiv`).
#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    dtos::feeds::{CreateFeed, FeedDto, IngestArxiv},
    ingest::{
        arxiv::{self, Filing, Ingested, Paper},
        rss::{self, PullReport},
    },
    models::feeds::{ActiveModel, Entity, Model},
};

#[debug_handler]
async fn list(State(ctx): State<AppContext>) -> Result<Json<Vec<FeedDto>>> {
    let all = Entity::listing().all(&ctx.db).await?;
    Ok(Json(all.into_iter().map(Into::into).collect()))
}

#[debug_handler]
async fn add(State(ctx): State<AppContext>, Json(params): Json<CreateFeed>) -> Result<Response> {
    let feed =
        ActiveModel::subscribe(&ctx.db, &params.url, params.area_id, params.project_id).await?;
    Ok((StatusCode::CREATED, Json(FeedDto::from(feed))).into_response())
}

#[debug_handler]
async fn pull(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Json<PullReport>> {
    let feed = Model::find(&ctx.db, id).await?;
    Ok(Json(rss::pull(&ctx.db, feed).await?))
}

#[debug_handler]
async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    Model::find(&ctx.db, id).await?.remove(&ctx.db).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[debug_handler]
async fn ingest_arxiv(
    State(ctx): State<AppContext>,
    Json(params): Json<IngestArxiv>,
) -> Result<Json<Vec<Ingested>>> {
    if params.ids.is_empty() {
        return Err(Error::BadRequest("ids cannot be empty".into()));
    }
    let papers = arxiv::fetch_ids(&params.ids).await?;
    let filing = Filing {
        area_id: params.area_id,
        project_id: params.project_id,
        note: params.note,
    };
    Ok(Json(arxiv::store(&ctx.db, papers, &filing).await?))
}

/// `?q=rogue waves&max=10`.
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub max: Option<u32>,
}

#[debug_handler]
async fn search_arxiv(Query(params): Query<SearchParams>) -> Result<Json<Vec<Paper>>> {
    let max = params.max.unwrap_or(10).clamp(1, 50);
    Ok(Json(arxiv::search(&params.q, max).await?))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/feeds", get(list))
        .add("/feeds", post(add))
        .add("/feeds/{id}/pull", post(pull))
        .add("/feeds/{id}", delete(remove))
        .add("/ingest/arxiv", post(ingest_arxiv))
        .add("/ingest/arxiv/search", get(search_arxiv))
}
