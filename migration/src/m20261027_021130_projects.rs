use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "projects",
            &[
                ("id", ColType::PkAuto),
                ("archived_at", ColType::TimestampWithTimeZoneNull),
                ("name", ColType::String),
                ("description", ColType::TextNull),
                ("status", ColType::StringNull),
            ],
            &[("area", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "projects").await
    }
}
