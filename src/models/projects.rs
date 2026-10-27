//! Model rules shared by the REST controllers and the `ztk` CLI.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::projects::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel, QueryOrder};

use super::areas;
use crate::dtos::projects::{CreateProject, UpdateProject};
pub type Projects = Entity;

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

// implement your read-oriented logic here
impl Model {
    /// Load a project or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    pub async fn update_with<C: ConnectionTrait>(
        self,
        db: &C,
        params: UpdateProject,
    ) -> ModelResult<Self> {
        let mut item = self.into_active_model();
        if let Some(name) = params.name {
            item.name = Set(name);
        }
        if params.description.is_some() {
            item.description = Set(params.description);
        }
        if params.status.is_some() {
            item.status = Set(params.status);
        }
        if let Some(area_id) = params.area_id {
            areas::Model::find(db, area_id).await?;
            item.area_id = Set(area_id);
        }
        Ok(item.update(db).await?)
    }

    /// Archive (`true`) or restore (`false`) the project.
    pub async fn set_archived<C: ConnectionTrait>(
        self,
        db: &C,
        archived: bool,
    ) -> ModelResult<Self> {
        let mut item = self.into_active_model();
        item.archived_at = Set(archived.then(|| chrono::Utc::now().into()));
        Ok(item.update(db).await?)
    }

    pub async fn remove<C: ConnectionTrait>(self, db: &C) -> ModelResult<()> {
        self.into_active_model().delete(db).await?;
        Ok(())
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateProject) -> ModelResult<Model> {
        areas::Model::find(db, params.area_id).await?;
        let item = Self {
            name: Set(params.name),
            description: Set(params.description),
            status: Set(params.status),
            area_id: Set(params.area_id),
            ..Default::default()
        };
        Ok(item.insert(db).await?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// Projects ordered by id, optionally scoped to an area; archived ones only
    /// when `archived` is true.
    #[must_use]
    pub fn listing(area_id: Option<i64>, archived: bool) -> Select<Self> {
        let mut q = if archived {
            Self::find().filter(Column::ArchivedAt.is_not_null())
        } else {
            Self::find().filter(Column::ArchivedAt.is_null())
        };
        if let Some(area_id) = area_id {
            q = q.filter(Column::AreaId.eq(area_id));
        }
        q.order_by_asc(Column::Id)
    }
}
