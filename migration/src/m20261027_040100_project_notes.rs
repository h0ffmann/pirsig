use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

/// A project pulls notes in by link; it never owns them. Archiving or deleting
/// the project drops the link, never the note.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "project_notes",
            &[("id", ColType::PkAuto), ("context", ColType::TextNull)],
            &[("project", ""), ("note", "")],
        )
        .await?;
        m.create_index(
            Index::create()
                .name("idx-project_notes-project_id-note_id")
                .table(Alias::new("project_notes"))
                .col(Alias::new("project_id"))
                .col(Alias::new("note_id"))
                .unique()
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "project_notes").await
    }
}
