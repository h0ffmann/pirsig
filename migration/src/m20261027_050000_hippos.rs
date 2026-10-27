use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

/// Hippo identities: a self-described way of thinking (ADHD, autistic,
/// neurotic, obsessive) whose notes form a rhizome instead of a tree.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "hippos",
            &[
                ("id", ColType::PkAuto),
                ("name", ColType::StringUniq),
                ("description", ColType::TextNull),
                // comma-separated, validated by the model
                ("traits", ColType::String),
                ("fixations", ColType::Text),
            ],
            &[],
        )
        .await?;
        // a plain column, as notes.resource_id: SQLite cannot add a foreign key
        // to an existing table, so `hippos::Model::remove` clears it
        add_column(m, "notes", "hippo_id", ColType::BigIntegerNull).await?;
        m.create_index(
            Index::create()
                .name("idx-notes-hippo_id")
                .table(Alias::new("notes"))
                .col(Alias::new("hippo_id"))
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_index(
            Index::drop()
                .name("idx-notes-hippo_id")
                .table(Alias::new("notes"))
                .to_owned(),
        )
        .await?;
        remove_column(m, "notes", "hippo_id").await?;
        drop_table(m, "hippos").await
    }
}
