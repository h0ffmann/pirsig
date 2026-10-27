use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "note_tags",
            &[("id", ColType::PkAuto), ("name", ColType::String)],
            &[("note", "")],
        )
        .await?;
        m.create_index(
            Index::create()
                .name("idx-note_tags-name")
                .table(Alias::new("note_tags"))
                .col(Alias::new("name"))
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "note_tags").await
    }
}
