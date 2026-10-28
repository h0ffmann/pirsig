//! RSS/Atom subscriptions: `psg rss add URL`, then `psg rss pull` turns new
//! entries into resources filed under the feed's area or project.
#![allow(clippy::missing_errors_doc)]
pub use super::_entities::feeds::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel, QueryOrder};

use super::{areas, projects, rules};
pub type Feeds = Entity;

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

impl Model {
    /// Load a feed or fail with [`ModelError::EntityNotFound`].
    pub async fn find<C: ConnectionTrait>(db: &C, id: i64) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// Record a successful pull, and the feed's own title when it had none.
    pub async fn fetched<C: ConnectionTrait>(
        self,
        db: &C,
        title: Option<String>,
    ) -> ModelResult<Self> {
        let keep = self.title.clone();
        let mut item = self.into_active_model();
        item.last_fetched_at = Set(Some(chrono::Utc::now().into()));
        item.title = Set(keep.or_else(|| rules::optional(title)));
        Ok(item.update(db).await?)
    }

    pub async fn remove<C: ConnectionTrait>(self, db: &C) -> ModelResult<()> {
        self.into_active_model().delete(db).await?;
        Ok(())
    }
}

impl ActiveModel {
    /// Subscribe to `url`; subscribing twice returns the existing feed.
    pub async fn subscribe<C: ConnectionTrait>(
        db: &C,
        url: &str,
        area_id: Option<i64>,
        project_id: Option<i64>,
    ) -> ModelResult<Model> {
        let url = url.trim();
        rules::url("url", url)?;
        if let Some(feed) = Entity::find().filter(Column::Url.eq(url)).one(db).await? {
            return Ok(feed);
        }
        if let Some(area_id) = area_id {
            areas::Model::find(db, area_id).await?;
        }
        if let Some(project_id) = project_id {
            projects::Model::find(db, project_id).await?;
        }
        Ok(Self {
            url: Set(url.to_string()),
            area_id: Set(area_id),
            project_id: Set(project_id),
            ..Default::default()
        }
        .insert(db)
        .await?)
    }
}

impl Entity {
    #[must_use]
    pub fn listing() -> Select<Self> {
        Self::find().order_by_asc(Column::Id)
    }
}
