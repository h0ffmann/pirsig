use std::path::PathBuf;

use clap::Subcommand;
use loco_rs::{app::AppContext, Result};

use super::Output;
use crate::vault::{self, ExportReport, ImportReport};

/// `psg obsidian export ./vault`, `psg obsidian import ./vault --dry-run`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Write the brain as a vault: PARA folders, notes flat in Zettelkasten/.
    Export {
        /// Vault folder; created if missing. Only files Pirsig wrote are replaced.
        dir: PathBuf,
        /// Also export archived areas, projects and resources, under Archive/.
        #[arg(long)]
        archived: bool,
    },
    /// Read a vault in: Pirsig PARA files by name, every other note by zid.
    Import {
        /// Vault folder.
        dir: PathBuf,
        /// Show what would happen; write nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

pub(crate) fn exported(r: &ExportReport, dir: &std::path::Path) -> String {
    format!(
        "exported {} notes, {} areas, {} projects, {} resources to {}{}",
        r.notes,
        r.areas,
        r.projects,
        r.resources,
        dir.display(),
        if r.removed > 0 {
            format!(" ({} stale files removed)", r.removed)
        } else {
            String::new()
        }
    )
}

pub(crate) fn imported(r: &ImportReport) -> String {
    let mut s =
        format!(
        "{}{} notes created, {} updated; {} areas, {} projects, {} tasks, {} resources; {} links",
        if r.dry_run { "dry run, nothing written: " } else { "" },
        r.notes_created,
        r.notes_updated,
        r.areas,
        r.projects,
        r.tasks,
        r.resources,
        r.links,
    );
    for w in &r.warnings {
        s.push_str("\n  ! ");
        s.push_str(w);
    }
    s
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    match args.action {
        Action::Export { dir, archived } => {
            let report = vault::export(&ctx.db, &dir, archived).await?;
            out.one(&report, |r| exported(r, &dir));
        }
        Action::Import { dir, dry_run } => {
            let report = vault::import(&ctx.db, &dir, dry_run).await?;
            out.one(&report, imported);
        }
    }
    Ok(())
}
