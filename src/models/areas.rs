//! Model rules shared by the REST controllers and the `ztk` CLI.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::areas::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{
    entity::prelude::*,
    sea_query::{ExprTrait, Func},
    ActiveValue::Set,
    IntoActiveModel, PaginatorTrait, QueryOrder, TransactionSession, TransactionTrait,
};

use super::{projects, resources, rules};
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
        let id = self.id;
        let mut item = self.into_active_model();
        if let Some(name) = params.name {
            item.name = Set(Entity::free_name(db, &name, Some(id)).await?);
        }
        if let Some(description) = params.description.into_option() {
            item.description = Set(rules::optional(description));
        }
        Ok(item.update(db).await?)
    }

    /// Archive (`true`) or restore (`false`) the area. Archiving is the A in PARA:
    /// the area is kept, but hidden from default listings. Archiving an area also
    /// archives its active projects and resources, so nothing active is left
    /// under it; restoring brings back only the area.
    pub async fn set_archived<C: ConnectionTrait + TransactionTrait>(
        self,
        db: &C,
        archived: bool,
    ) -> ModelResult<Self> {
        let txn = db.begin().await?;
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        if archived {
            projects::Entity::update_many()
                .col_expr(projects::Column::ArchivedAt, Expr::value(now))
                .filter(projects::Column::AreaId.eq(self.id))
                .filter(projects::Column::ArchivedAt.is_null())
                .exec(&txn)
                .await?;
            resources::Entity::update_many()
                .col_expr(resources::Column::ArchivedAt, Expr::value(now))
                .filter(resources::Column::AreaId.eq(self.id))
                .filter(resources::Column::ArchivedAt.is_null())
                .exec(&txn)
                .await?;
        }
        let mut item = self.into_active_model();
        item.archived_at = Set(archived.then_some(now));
        let area = item.update(&txn).await?;
        txn.commit().await?;
        Ok(area)
    }

    /// Delete the area. With projects under it this is refused unless `force`,
    /// which deletes them and their tasks too (archive the area instead to keep
    /// them). Resources and feeds filed under it are kept, unfiled.
    pub async fn remove<C: ConnectionTrait>(self, db: &C, force: bool) -> ModelResult<()> {
        let children = projects::Entity::find()
            .filter(projects::Column::AreaId.eq(self.id))
            .count(db)
            .await?;
        if children > 0 && !force {
            return Err(rules::invalid(
                "force",
                &format!(
                    "area #{} has {children} project(s); archive it, or force the delete to remove them too",
                    self.id
                ),
            ));
        }
        self.into_active_model().delete(db).await?;
        Ok(())
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn create<C: ConnectionTrait>(db: &C, params: CreateArea) -> ModelResult<Model> {
        let item = Self {
            name: Set(Entity::free_name(db, &params.name, None).await?),
            description: Set(rules::optional(params.description)),
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

    /// `name`, trimmed, if no other area has it (case-insensitive): two areas
    /// with one name are one responsibility filed twice.
    async fn free_name<C: ConnectionTrait>(
        db: &C,
        name: &str,
        except: Option<i64>,
    ) -> ModelResult<String> {
        let name = rules::text("name", name)?;
        let mut q = Self::find().filter(ExprTrait::eq(
            Func::lower(Expr::col(Column::Name)),
            name.to_lowercase(),
        ));
        if let Some(id) = except {
            q = q.filter(Column::Id.ne(id));
        }
        if q.count(db).await? > 0 {
            return Err(rules::invalid(
                "name",
                &format!("an area named \"{name}\" already exists"),
            ));
        }
        Ok(name)
    }
}
