use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

/// RSS/Atom subscriptions. Each pulled entry becomes a resource, filed under
/// the feed's area or project.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "feeds",
            &[
                ("id", ColType::PkAuto),
                ("url", ColType::StringUniq),
                ("title", ColType::StringNull),
                ("last_fetched_at", ColType::TimestampWithTimeZoneNull),
            ],
            &[("area?", ""), ("project?", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "feeds").await
    }
}
