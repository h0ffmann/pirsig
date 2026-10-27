use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "resources",
            &[
                ("id", ColType::PkAuto),
                ("archived_at", ColType::TimestampWithTimeZoneNull),
                ("title", ColType::String),
                ("url", ColType::StringNull),
                ("body", ColType::TextNull),
            ],
            &[("area?", ""), ("project?", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "resources").await
    }
}
