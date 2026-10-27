use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "notes",
            &[
                ("id", ColType::PkAuto),
                ("zid", ColType::StringUniq),
                ("title", ColType::String),
                ("body", ColType::Text),
                ("kind", ColType::String),
                ("source", ColType::StringNull),
            ],
            &[],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "notes").await
    }
}
