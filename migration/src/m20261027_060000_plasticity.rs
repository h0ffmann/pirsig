use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

/// Plasticity: plasts (exercise kinds) with each one's level and schedule, and
/// every attempt at them.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "plasts",
            &[
                ("id", ColType::PkAuto),
                ("slug", ColType::StringUniq),
                ("domain", ColType::String),
                ("title", ColType::String),
                ("description", ColType::TextNull),
                // "ztk" for the catalog; "user" is reserved for the person's own
                ("origin", ColType::String),
                ("level", ColType::Integer),
                ("streak", ColType::Integer),
                ("due_at", ColType::TimestampWithTimeZoneNull),
                ("practiced_at", ColType::TimestampWithTimeZoneNull),
            ],
            &[],
        )
        .await?;
        create_table(
            m,
            "plast_attempts",
            &[
                ("id", ColType::PkAuto),
                ("seed", ColType::BigInteger),
                ("level", ColType::Integer),
                ("prompt", ColType::Text),
                ("expected", ColType::String),
                ("answer", ColType::String),
                ("correct", ColType::Boolean),
                ("millis", ColType::BigIntegerNull),
            ],
            &[("plast", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "plast_attempts").await?;
        drop_table(m, "plasts").await
    }
}
