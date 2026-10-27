//! Model rules shared by the REST controllers and the `ztk` CLI.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::tasks::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel, QueryOrder};

use super::{projects, rules};
use crate::dtos::tasks::{CreateTask, UpdateTask};
pub type Tasks = Entity;

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

/// The project a task goes into: it must exist and not be archived.
async fn open_project<C: ConnectionTrait>(db: &C, project_id: i64) -> ModelResult<()> {
    let project = projects::Model::find(db, project_id).await?;
    if project.archived_at.is_some() {
        return Err(rules::invalid(
            "project_id",
            &format!("project #{project_id} is archived; unarchive it first"),
        ));
    }
    Ok(())
}

// implement your read-oriented logic here
impl Model {
    /// Load a task or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    pub async fn update_with<C: ConnectionTrait>(
        self,
        db: &C,
        params: UpdateTask,
    ) -> ModelResult<Self> {
        let mut item = self.into_active_model();
        if let Some(title) = params.title {
            item.title = Set(rules::text("title", &title)?);
        }
        if let Some(notes) = params.notes.into_option() {
            item.notes = Set(rules::optional(notes));
        }
        if let Some(done) = params.done {
            item.done = Set(done);
        }
        if let Some(due_on) = params.due_on.into_option() {
            item.due_on = Set(due_on);
        }
        if let Some(project_id) = params.project_id {
            open_project(db, project_id).await?;
            item.project_id = Set(project_id);
        }
        Ok(item.update(db).await?)
    }

    pub async fn remove<C: ConnectionTrait>(self, db: &C) -> ModelResult<()> {
        self.into_active_model().delete(db).await?;
        Ok(())
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateTask) -> ModelResult<Model> {
        open_project(db, params.project_id).await?;
        let item = Self {
            title: Set(rules::text("title", &params.title)?),
            notes: Set(rules::optional(params.notes)),
            done: Set(params.done),
            due_on: Set(params.due_on),
            project_id: Set(params.project_id),
            ..Default::default()
        };
        Ok(item.insert(db).await?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// Tasks ordered by id, optionally scoped to a project and/or done state.
    #[must_use]
    pub fn listing(project_id: Option<i64>, done: Option<bool>) -> Select<Self> {
        let mut q = Self::find();
        if let Some(project_id) = project_id {
            q = q.filter(Column::ProjectId.eq(project_id));
        }
        if let Some(done) = done {
            q = q.filter(Column::Done.eq(done));
        }
        q.order_by_asc(Column::Id)
    }
}
