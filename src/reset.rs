//! Starting a brain over: a backup of the database, then an empty brain.
//!
//! `psg gh --overwrite` and `psg fork import --overwrite` use this to replace
//! a brain (the fork state that came with the repository, or a brain grown
//! before) with another one. Nothing is lost: the database is copied first.
#![allow(clippy::missing_errors_doc)]
use std::path::PathBuf;

use loco_rs::{Error, Result};
use sea_orm::{
    sea_query::Expr, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    TransactionTrait,
};
use serde::Serialize;

use crate::models::_entities::{
    areas, feeds, hippos, note_links, note_tags, notes, plast_attempts, plasts, project_notes,
    projects, resources, tasks,
};

/// What a wipe removed.
#[derive(Debug, Default, Serialize)]
pub struct Wiped {
    /// Where the brain was copied first.
    pub backup: Option<PathBuf>,
    pub notes: u64,
    pub areas: u64,
    pub projects: u64,
    pub tasks: u64,
    pub resources: u64,
    pub hippos: u64,
    pub feeds: u64,
}

/// Whether the brain holds anything a wipe would remove.
pub async fn is_empty(db: &DatabaseConnection) -> Result<bool> {
    Ok(notes::Entity::find().count(db).await? == 0
        && areas::Entity::find().count(db).await? == 0
        && resources::Entity::find().count(db).await? == 0
        && hippos::Entity::find().count(db).await? == 0)
}

/// The database file behind `uri` (`sqlite://brain.sqlite?mode=rwc`), when it is one.
fn sqlite_file(uri: &str) -> Option<PathBuf> {
    let path = uri.strip_prefix("sqlite://")?;
    let path = path.split('?').next().unwrap_or(path);
    (!path.is_empty() && !path.contains(":memory:")).then(|| PathBuf::from(path))
}

/// Copy the brain next to its database file, as
/// `<name>.backup-<timestamp>.sqlite`. `VACUUM INTO` copies a consistent state
/// even while the brain is open.
pub async fn backup(db: &DatabaseConnection, uri: &str) -> Result<PathBuf> {
    let file = sqlite_file(uri).ok_or_else(|| {
        Error::Message(
            "only a SQLite brain can be backed up before it is replaced; back it up by hand".into(),
        )
    })?;
    let stem = file
        .file_stem()
        .map_or_else(|| "brain".into(), |s| s.to_string_lossy().into_owned());
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let to = file.with_file_name(format!("{stem}.backup-{stamp}.sqlite"));
    let quoted = to.display().to_string().replace('\'', "''");
    db.execute_unprepared(&format!("VACUUM INTO '{quoted}'"))
        .await?;
    Ok(to)
}

/// Back the brain up, then remove everything in it: notes and their links and
/// tags, PARA, Hippos, feeds, and the Plasticity progress (the exercises
/// stay). Accounts are kept.
pub async fn wipe(db: &DatabaseConnection, uri: &str) -> Result<Wiped> {
    let backup = backup(db, uri).await?;
    let txn = db.begin().await?;
    project_notes::Entity::delete_many().exec(&txn).await?;
    note_links::Entity::delete_many().exec(&txn).await?;
    note_tags::Entity::delete_many().exec(&txn).await?;
    let notes = notes::Entity::delete_many().exec(&txn).await?.rows_affected;
    let tasks = tasks::Entity::delete_many().exec(&txn).await?.rows_affected;
    let feeds = feeds::Entity::delete_many().exec(&txn).await?.rows_affected;
    let resources = resources::Entity::delete_many()
        .exec(&txn)
        .await?
        .rows_affected;
    let projects = projects::Entity::delete_many()
        .exec(&txn)
        .await?
        .rows_affected;
    let areas = areas::Entity::delete_many().exec(&txn).await?.rows_affected;
    let hippos = hippos::Entity::delete_many()
        .exec(&txn)
        .await?
        .rows_affected;
    plast_attempts::Entity::delete_many().exec(&txn).await?;
    plasts::Entity::update_many()
        .col_expr(plasts::Column::Level, Expr::value(1))
        .col_expr(plasts::Column::Streak, Expr::value(0))
        .col_expr(plasts::Column::DueAt, Expr::cust("NULL"))
        .col_expr(plasts::Column::PracticedAt, Expr::cust("NULL"))
        .exec(&txn)
        .await?;
    txn.commit().await?;
    Ok(Wiped {
        backup: Some(backup),
        notes,
        areas,
        projects,
        tasks,
        resources,
        hippos,
        feeds,
    })
}

/// One line for a wipe, for the CLI.
#[must_use]
pub fn wiped_line(w: &Wiped) -> String {
    format!(
        "backed up to {}; removed {} notes, {} areas, {} projects, {} tasks, {} resources, {} Hippos, {} feeds",
        w.backup
            .as_ref()
            .map_or_else(|| "nowhere".into(), |p| p.display().to_string()),
        w.notes,
        w.areas,
        w.projects,
        w.tasks,
        w.resources,
        w.hippos,
        w.feeds
    )
}

#[cfg(test)]
mod tests {
    use super::sqlite_file;

    #[test]
    fn finds_the_sqlite_file() {
        assert_eq!(
            sqlite_file("sqlite://zettelkasten_development.sqlite?mode=rwc"),
            Some("zettelkasten_development.sqlite".into())
        );
        assert_eq!(sqlite_file("sqlite::memory:"), None);
        assert_eq!(sqlite_file("postgres://localhost/brain"), None);
    }
}
