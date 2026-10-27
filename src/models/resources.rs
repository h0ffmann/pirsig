//! Model rules shared by the REST controllers and the `ztk` CLI.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::resources::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel, QueryOrder};

use super::{areas, projects};
use crate::dtos::resources::{CreateResource, UpdateResource};
pub type Resources = Entity;

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
    /// Load a resource or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    pub async fn update_with<C: ConnectionTrait>(
        self,
        db: &C,
        params: UpdateResource,
    ) -> ModelResult<Self> {
        let mut item = self.into_active_model();
        if let Some(title) = params.title {
            item.title = Set(title);
        }
        if params.url.is_some() {
            item.url = Set(params.url);
        }
        if params.body.is_some() {
            item.body = Set(params.body);
        }
        if let Some(area_id) = params.area_id {
            areas::Model::find(db, area_id).await?;
            item.area_id = Set(Some(area_id));
        }
        if let Some(project_id) = params.project_id {
            projects::Model::find(db, project_id).await?;
            item.project_id = Set(Some(project_id));
        }
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

    pub async fn remove<C: ConnectionTrait>(self, db: &C) -> ModelResult<()> {
        self.into_active_model().delete(db).await?;
        Ok(())
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    /// A resource may stand alone, or hang off an area and/or a project.
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateResource) -> ModelResult<Model> {
        if let Some(area_id) = params.area_id {
            areas::Model::find(db, area_id).await?;
        }
        if let Some(project_id) = params.project_id {
            projects::Model::find(db, project_id).await?;
        }
        let item = Self {
            title: Set(params.title),
            url: Set(params.url),
            body: Set(params.body),
            area_id: Set(params.area_id),
            project_id: Set(params.project_id),
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
}
