//! Model rules shared by the REST controllers and the `psg` CLI.
//!
//! A resource is the PARA home of a source: its literature notes point at it
//! (`notes.resource_id`), and its `citekey` is how they cite it.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::resources::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{
    entity::prelude::*, ActiveValue::Set, IntoActiveModel, PaginatorTrait, QueryOrder,
    TransactionSession, TransactionTrait,
};

use super::{areas, notes, projects, rules};
use crate::dtos::resources::{CreateResource, UpdateResource};
pub type Resources = Entity;

pub const KINDS: [&str; 7] = ["article", "book", "paper", "repo", "video", "web", "other"];

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

fn kind(value: Option<String>) -> ModelResult<Option<String>> {
    let value = rules::optional(value).map(|k| k.to_lowercase());
    if let Some(k) = &value {
        rules::one_of("kind", k, &KINDS)?;
    }
    Ok(value)
}

fn url(value: Option<String>) -> ModelResult<Option<String>> {
    let value = rules::optional(value);
    if let Some(u) = &value {
        rules::url("url", u)?;
    }
    Ok(value)
}

/// A BibTeX-safe key, free in the brain: `vaswani2017attention`.
async fn citekey<C: ConnectionTrait>(
    db: &C,
    value: Option<String>,
    except: Option<i64>,
) -> ModelResult<Option<String>> {
    let Some(key) = rules::optional(value) else {
        return Ok(None);
    };
    let key = key.trim_start_matches('@').to_string();
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':' | '.'))
    {
        return Err(rules::invalid(
            "citekey",
            "citekey takes letters, digits and _ - : . only",
        ));
    }
    let mut q = Entity::find().filter(Column::Citekey.eq(&key));
    if let Some(id) = except {
        q = q.filter(Column::Id.ne(id));
    }
    if q.count(db).await? > 0 {
        return Err(rules::invalid(
            "citekey",
            &format!("citekey @{key} is already taken"),
        ));
    }
    Ok(Some(key))
}

/// Area and project must exist, and when both are given the project must be in
/// that area: a resource cannot be filed in two places that disagree.
async fn placement<C: ConnectionTrait>(
    db: &C,
    area_id: Option<i64>,
    project_id: Option<i64>,
) -> ModelResult<()> {
    if let Some(area_id) = area_id {
        areas::Model::find(db, area_id).await?;
    }
    if let Some(project_id) = project_id {
        let project = projects::Model::find(db, project_id).await?;
        if area_id.is_some_and(|a| a != project.area_id) {
            return Err(rules::invalid(
                "area_id",
                &format!(
                    "project #{project_id} is in area #{}, not #{}",
                    project.area_id,
                    area_id.unwrap_or_default()
                ),
            ));
        }
    }
    Ok(())
}

// implement your read-oriented logic here
impl Model {
    /// Load a resource or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// How a literature note names this source: `@citekey`, else the URL, else the title.
    #[must_use]
    pub fn cite(&self) -> String {
        self.citekey.as_ref().map_or_else(
            || self.url.clone().unwrap_or_else(|| self.title.clone()),
            |k| format!("@{k}"),
        )
    }

    pub async fn update_with<C: ConnectionTrait>(
        self,
        db: &C,
        params: UpdateResource,
    ) -> ModelResult<Self> {
        let (area, project) = (
            params.area_id.into_option(),
            params.project_id.into_option(),
        );
        let area_id = area.unwrap_or(self.area_id);
        let project_id = project.unwrap_or(self.project_id);
        if area.is_some() || project.is_some() {
            placement(db, area_id, project_id).await?;
        }
        let id = self.id;
        let mut item = self.into_active_model();
        if let Some(title) = params.title {
            item.title = Set(rules::text("title", &title)?);
        }
        if let Some(value) = params.url.into_option() {
            item.url = Set(url(value)?);
        }
        if let Some(body) = params.body.into_option() {
            item.body = Set(rules::optional(body));
        }
        if let Some(value) = params.kind.into_option() {
            item.kind = Set(kind(value)?);
        }
        if let Some(value) = params.citekey.into_option() {
            item.citekey = Set(citekey(db, value, Some(id)).await?);
        }
        item.area_id = Set(area_id);
        item.project_id = Set(project_id);
        Ok(item.update(db).await?)
    }

    /// Archive (`true`) or restore (`false`) the resource.
    pub async fn set_archived<C: ConnectionTrait>(
        self,
        db: &C,
        archived: bool,
    ) -> ModelResult<Self> {
        let mut item = self.into_active_model();
        item.archived_at = Set(archived.then(|| chrono::Utc::now().into()));
        Ok(item.update(db).await?)
    }

    /// Delete the resource. Its literature notes stay in the slipbox: they keep
    /// their `source` and lose only the pointer here.
    pub async fn remove<C: ConnectionTrait + TransactionTrait>(self, db: &C) -> ModelResult<()> {
        let txn = db.begin().await?;
        notes::Entity::update_many()
            .col_expr(notes::Column::ResourceId, Expr::value(Option::<i64>::None))
            .filter(notes::Column::ResourceId.eq(self.id))
            .exec(&txn)
            .await?;
        self.into_active_model().delete(&txn).await?;
        txn.commit().await?;
        Ok(())
    }

    /// The literature notes written from this resource.
    pub async fn notes<C: ConnectionTrait>(&self, db: &C) -> ModelResult<Vec<notes::Model>> {
        Ok(notes::Entity::find()
            .filter(notes::Column::ResourceId.eq(self.id))
            .order_by_asc(notes::Column::Id)
            .all(db)
            .await?)
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    /// A resource may stand alone, or hang off an area and/or a project.
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateResource) -> ModelResult<Model> {
        placement(db, params.area_id, params.project_id).await?;
        let item = Self {
            title: Set(rules::text("title", &params.title)?),
            url: Set(url(params.url)?),
            body: Set(rules::optional(params.body)),
            area_id: Set(params.area_id),
            project_id: Set(params.project_id),
            kind: Set(kind(params.kind)?),
            citekey: Set(citekey(db, params.citekey, None).await?),
            external_id: Set(rules::optional(params.external_id)),
            ..Default::default()
        };
        Ok(item.insert(db).await?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// Resources ordered by id, optionally scoped to an area or project;
    /// archived ones only when `archived` is true.
    #[must_use]
    pub fn listing(area_id: Option<i64>, project_id: Option<i64>, archived: bool) -> Select<Self> {
        let mut q = if archived {
            Self::find().filter(Column::ArchivedAt.is_not_null())
        } else {
            Self::find().filter(Column::ArchivedAt.is_null())
        };
        if let Some(area_id) = area_id {
            q = q.filter(Column::AreaId.eq(area_id));
        }
        if let Some(project_id) = project_id {
            q = q.filter(Column::ProjectId.eq(project_id));
        }
        q.order_by_asc(Column::Id)
    }

    /// The resource an ingest already made for `external_id`, if any.
    pub async fn by_external_id<C: ConnectionTrait>(
        db: &C,
        external_id: &str,
    ) -> ModelResult<Option<Model>> {
        Ok(Self::find()
            .filter(Column::ExternalId.eq(external_id))
            .one(db)
            .await?)
    }
}
