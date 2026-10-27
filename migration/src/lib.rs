#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20261027_020951_areas;
mod m20261027_021130_projects;
mod m20261027_021308_tasks;
mod m20261027_021525_resources;
mod m20261027_030000_notes;
mod m20261027_030100_note_links;
mod m20261027_030200_note_tags;
mod m20261027_040000_para_methodology;
mod m20261027_040100_project_notes;
mod m20261027_040200_feeds;
mod m20261027_050000_hippos;
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
            Box::new(m20261027_030000_notes::Migration),
            Box::new(m20261027_030100_note_links::Migration),
            Box::new(m20261027_030200_note_tags::Migration),
            Box::new(m20261027_040000_para_methodology::Migration),
            Box::new(m20261027_040100_project_notes::Migration),
            Box::new(m20261027_040200_feeds::Migration),
            Box::new(m20261027_050000_hippos::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
