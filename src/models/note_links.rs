//! Links between notes.
//!
//! Auto links come from `[[wikilinks]]` in a note's body
//! and are rebuilt on every save; hand-made links (`auto = false`) carry the
//! reason two ideas belong together and are only changed on purpose.
//! The rules live on [`super::notes::Model`].
pub use super::_entities::note_links::{ActiveModel, Column, Entity, Model};
use sea_orm::entity::prelude::*;
pub type NoteLinks = Entity;

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
