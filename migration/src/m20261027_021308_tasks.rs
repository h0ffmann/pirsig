use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "tasks",
            &[
                ("id", ColType::PkAuto),
                ("title", ColType::String),
                ("notes", ColType::TextNull),
                ("done", ColType::Boolean),
                ("due_on", ColType::DateNull),
            ],
            &[("project", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "tasks").await
    }
}
