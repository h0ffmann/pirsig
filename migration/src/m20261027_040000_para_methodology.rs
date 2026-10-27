use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

/// What PARA and the slipbox need from each other: a project's deadline, a
/// resource's citation (`citekey`) and ingest identity (`external_id`), and the
/// resource a literature note was written from.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column(m, "projects", "due_on", ColType::DateNull).await?;
        add_column(m, "resources", "kind", ColType::StringNull).await?;
        add_column(m, "resources", "citekey", ColType::StringNull).await?;
        add_column(m, "resources", "external_id", ColType::StringNull).await?;
        // a plain column: SQLite cannot add a foreign key to an existing table, so
        // the model clears it when the resource goes (see `resources::Model::remove`)
        add_column(m, "notes", "resource_id", ColType::BigIntegerNull).await?;

        for (table, col, unique) in [
            ("resources", "citekey", true),
            ("resources", "external_id", true),
            ("notes", "resource_id", false),
        ] {
            let mut idx = Index::create()
                .name(format!("idx-{table}-{col}"))
                .table(Alias::new(table))
                .col(Alias::new(col))
                .to_owned();
            if unique {
                idx.unique();
            }
            m.create_index(idx).await?;
        }

        // every project has a status from now on; the old default was "none"
        m.exec_stmt(
            Query::update()
                .table(Alias::new("projects"))
                .value(Alias::new("status"), "active")
                .and_where(Expr::col(Alias::new("status")).is_null())
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for (table, col) in [
            ("resources", "citekey"),
            ("resources", "external_id"),
            ("notes", "resource_id"),
        ] {
            m.drop_index(
                Index::drop()
                    .name(format!("idx-{table}-{col}"))
                    .table(Alias::new(table))
                    .to_owned(),
            )
            .await?;
        }
        remove_column(m, "notes", "resource_id").await?;
        remove_column(m, "resources", "external_id").await?;
        remove_column(m, "resources", "citekey").await?;
        remove_column(m, "resources", "kind").await?;
        remove_column(m, "projects", "due_on").await
    }
}
