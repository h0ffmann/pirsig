//! Model rules shared by the REST controllers and the `ztk` CLI.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::projects::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{
    entity::prelude::*, ActiveValue::Set, IntoActiveModel, PaginatorTrait, QueryOrder,
    TransactionSession, TransactionTrait,
};

use super::{areas, notes, project_notes, resources, rules, tasks};
use crate::dtos::projects::{CreateProject, UpdateProject};
pub type Projects = Entity;

/// Where a project stands. `done` projects still show until they are archived.
pub const STATUSES: [&str; 4] = ["active", "waiting", "someday", "done"];
pub const DEFAULT_STATUS: &str = "active";

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

/// `status`, trimmed and lowercased, if it is one of [`STATUSES`].
fn status(value: &str) -> ModelResult<String> {
    let value = value.trim().to_lowercase();
    rules::one_of("status", &value, &STATUSES)?;
    Ok(value)
}

/// The area a project goes into: it must exist and not be archived.
async fn open_area<C: ConnectionTrait>(db: &C, area_id: i64) -> ModelResult<areas::Model> {
    let area = areas::Model::find(db, area_id).await?;
    if area.archived_at.is_some() {
        return Err(rules::invalid(
            "area_id",
            &format!("area #{area_id} is archived; unarchive it first or choose another area"),
        ));
    }
    Ok(area)
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

    pub async fn update_with<C: ConnectionTrait + TransactionTrait>(
        self,
        db: &C,
        params: UpdateProject,
    ) -> ModelResult<Self> {
        let txn = db.begin().await?;
        let id = self.id;
        let mut item = self.into_active_model();
        if let Some(name) = params.name {
            item.name = Set(rules::text("name", &name)?);
        }
        if let Some(description) = params.description.into_option() {
            item.description = Set(rules::optional(description));
        }
        if let Some(value) = params.status {
            item.status = Set(Some(status(&value)?));
        }
        if let Some(due_on) = params.due_on.into_option() {
            item.due_on = Set(due_on);
        }
        if let Some(area_id) = params.area_id {
            open_area(&txn, area_id).await?;
            item.area_id = Set(area_id);
            // a resource filed under both the project and its old area follows the project
            resources::Entity::update_many()
                .col_expr(resources::Column::AreaId, Expr::value(area_id))
                .filter(resources::Column::ProjectId.eq(id))
                .filter(resources::Column::AreaId.is_not_null())
                .exec(&txn)
                .await?;
        }
        let project = item.update(&txn).await?;
        txn.commit().await?;
        Ok(project)
    }

    /// Archive (`true`) or restore (`false`) the project. A project cannot come
    /// back while its area is archived: nothing active lives in the archive.
    pub async fn set_archived<C: ConnectionTrait>(
        self,
        db: &C,
        archived: bool,
    ) -> ModelResult<Self> {
        if !archived {
            open_area(db, self.area_id).await?;
        }
        let mut item = self.into_active_model();
        item.archived_at = Set(archived.then(|| chrono::Utc::now().into()));
        Ok(item.update(db).await?)
    }

    /// Delete the project. With tasks this is refused unless `force`, which
    /// deletes them too. Its notes stay in the slipbox, only unlinked from it;
    /// its resources stay, unfiled.
    pub async fn remove<C: ConnectionTrait>(self, db: &C, force: bool) -> ModelResult<()> {
        let children = tasks::Entity::find()
            .filter(tasks::Column::ProjectId.eq(self.id))
            .count(db)
            .await?;
        if children > 0 && !force {
            return Err(rules::invalid(
                "force",
                &format!(
                    "project #{} has {children} task(s); archive it, or force the delete to remove them too",
                    self.id
                ),
            ));
        }
        self.into_active_model().delete(db).await?;
        Ok(())
    }

    /// Pull a note into the project, saying why. Linking twice updates the reason.
    pub async fn link_note<C: ConnectionTrait>(
        &self,
        db: &C,
        note: &notes::Model,
        context: Option<String>,
    ) -> ModelResult<project_notes::Model> {
        let context = rules::optional(context);
        let existing = project_notes::Entity::find()
            .filter(project_notes::Column::ProjectId.eq(self.id))
            .filter(project_notes::Column::NoteId.eq(note.id))
            .one(db)
            .await?;
        Ok(match existing {
            Some(link) => {
                let mut link = link.into_active_model();
                link.context = Set(context);
                link.update(db).await?
            }
            None => {
                project_notes::ActiveModel {
                    project_id: Set(self.id),
                    note_id: Set(note.id),
                    context: Set(context),
                    ..Default::default()
                }
                .insert(db)
                .await?
            }
        })
    }

    /// Drop a note from the project; returns whether it was there. The note stays.
    pub async fn unlink_note<C: ConnectionTrait>(
        &self,
        db: &C,
        note: &notes::Model,
    ) -> ModelResult<bool> {
        let res = project_notes::Entity::delete_many()
            .filter(project_notes::Column::ProjectId.eq(self.id))
            .filter(project_notes::Column::NoteId.eq(note.id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// The notes this project pulls in, with why, in the order they were added.
    pub async fn notes<C: ConnectionTrait>(
        &self,
        db: &C,
    ) -> ModelResult<Vec<(notes::Model, Option<String>)>> {
        let links = project_notes::Entity::find()
            .filter(project_notes::Column::ProjectId.eq(self.id))
            .order_by_asc(project_notes::Column::Id)
            .find_also_related(notes::Entity)
            .all(db)
            .await?;
        Ok(links
            .into_iter()
            .filter_map(|(link, note)| note.map(|n| (n, link.context)))
            .collect())
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateProject) -> ModelResult<Model> {
        open_area(db, params.area_id).await?;
        let item = Self {
            name: Set(rules::text("name", &params.name)?),
            description: Set(rules::optional(params.description)),
            status: Set(Some(status(
                params.status.as_deref().unwrap_or(DEFAULT_STATUS),
            )?)),
            area_id: Set(params.area_id),
            due_on: Set(params.due_on),
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
