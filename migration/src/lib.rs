#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20261027_020951_areas;
mod m20261027_021130_projects;
mod m20261027_021308_tasks;
mod m20261027_021525_resources;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20261027_020951_areas::Migration),
            Box::new(m20261027_021130_projects::Migration),
            Box::new(m20261027_021308_tasks::Migration),
            Box::new(m20261027_021525_resources::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
