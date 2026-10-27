//! Model rules shared by the REST controllers and the `ztk` CLI.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::areas::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel, QueryOrder};

use crate::dtos::areas::{CreateArea, UpdateArea};
pub type Areas = Entity;

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
    /// Load an area or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    pub async fn update_with<C: ConnectionTrait>(
        self,
        db: &C,
        params: UpdateArea,
    ) -> ModelResult<Self> {
        let mut item = self.into_active_model();
        if let Some(name) = params.name {
            item.name = Set(name);
        }
        if params.description.is_some() {
            item.description = Set(params.description);
        }
        Ok(item.update(db).await?)
    }

    /// Archive (`true`) or restore (`false`) the area. Archiving is the A in PARA:
    /// the area is kept, but hidden from default listings.
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
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateArea) -> ModelResult<Model> {
        let item = Self {
            name: Set(params.name),
            description: Set(params.description),
            ..Default::default()
        };
        Ok(item.insert(db).await?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// Areas ordered by id; archived ones only when `archived` is true.
    #[must_use]
    pub fn listing(archived: bool) -> Select<Self> {
        let q = if archived {
            Self::find().filter(Column::ArchivedAt.is_not_null())
        } else {
            Self::find().filter(Column::ArchivedAt.is_null())
        };
        q.order_by_asc(Column::Id)
    }
}
