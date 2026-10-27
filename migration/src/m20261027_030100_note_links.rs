use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "note_links",
            &[
                ("id", ColType::PkAuto),
                ("context", ColType::TextNull),
                ("auto", ColType::Boolean),
            ],
            &[("note", "source_id"), ("note", "target_id")],
        )
        .await?;
        // backlinks and graph walks look links up from both ends
        for col in ["source_id", "target_id"] {
            m.create_index(
                Index::create()
                    .name(format!("idx-note_links-{col}"))
                    .table(Alias::new("note_links"))
                    .col(Alias::new(col))
                    .to_owned(),
            )
            .await?;
        }
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "note_links").await
    }
}
