use std::fmt::Write;

use clap::{Subcommand, ValueEnum};
use loco_rs::{app::AppContext, Result};
use pirsig_github::{
    index::{plan, Depth},
    scope::{Private, Scope},
};
use sea_orm::TransactionTrait;

use super::{explain, Output};
use crate::{
    ingest::github::{self, Client, Filing, Indexed, Repo, Tiering},
    reset,
};

/// `psg bootstrap gh`, `psg bootstrap gh --private minimal --readme --dry-run`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    source: Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Forks {
    /// One "GitHub: public forks" note, a line per fork.
    Index,
    /// A note per fork, as for own repositories.
    Notes,
    /// Leave forks out.
    Skip,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PrivateRepos {
    /// Leave them out.
    Skip,
    /// Name, URL, language and last push; never description, topics or README.
    Minimal,
}

#[derive(Debug, Subcommand)]
enum Source {
    /// Your GitHub repositories and public forks: a resource and a literature
    /// note each, categorised and indexed in hub notes. Uses `GH_TOKEN` or
    /// `GITHUB_TOKEN` (e.g. `GH_TOKEN=$(gh auth token)`); without one, only a named
    /// user's public repos, within GitHub's anonymous rate limit.
    Gh(GhArgs),
}

/// `psg gh` and `psg bootstrap gh`.
#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct GhArgs {
    /// GitHub user (default: the token's user).
    user: Option<String>,
    /// Your private repositories (only seen with your own token).
    #[arg(long, value_enum, default_value = "skip")]
    private: PrivateRepos,
    /// Public forks: one index note (index), a note each (notes), or none (skip).
    #[arg(long, value_enum, default_value = "index")]
    forks: Forks,
    /// Own repositories with no push in this many months go in one cold index
    /// note instead of a note each.
    #[arg(long, value_name = "MONTHS", default_value_t = 24)]
    cold_after: u32,
    /// Also read each public repo's README for its first paragraph.
    #[arg(long)]
    readme: bool,
    /// File here the repos whose category has no area in the brain.
    #[arg(short, long, value_name = "AREA_ID")]
    area: Option<i64>,
    /// Show what would be indexed, and write nothing.
    #[arg(long)]
    dry_run: bool,
    /// Replace the whole brain (the fork state it came with, or your own) with
    /// your GitHub: the database is backed up next to itself, emptied, then
    /// filled from GitHub.
    #[arg(long, conflicts_with = "dry_run")]
    overwrite: bool,
}

fn indexed_line(i: &Indexed) -> String {
    let what = match (i.created, i.updated) {
        (true, _) => "added",
        (false, true) => "updated",
        (false, false) => "known",
    };
    let id = i
        .resource_id
        .map(|id| format!(" #{id}"))
        .unwrap_or_default();
    let mut s = format!("{what}{id} {} repo/{}", i.repo, i.category);
    if i.fork {
        s.push_str(" fork");
    }
    if i.private {
        s.push_str(" private");
    }
    if i.depth == Depth::Minimal {
        s.push_str(" (minimal)");
    }
    match &i.area {
        Some(a) => {
            let _ = write!(s, " [{a}]");
        }
        None => s.push_str(" [unfiled]"),
    }
    if let Some(index) = i.index {
        let _ = write!(s, " → {index}");
        return s;
    }
    if let Some(z) = &i.note {
        let _ = write!(s, "; note {z}");
    }
    s
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let Source::Gh(gh) = args.source;
    run_gh(ctx, out, gh).await
}

pub async fn run_gh(ctx: &AppContext, out: &Output, args: GhArgs) -> Result<()> {
    let GhArgs {
        user,
        private,
        forks,
        cold_after,
        readme,
        area,
        dry_run,
        overwrite,
    } = args;
    let client = Client::from_env()?;
    let (user, facts) = client.facts(user.as_deref()).await?;
    let scope = Scope {
        forks: forks != Forks::Skip,
        own_private: match private {
            PrivateRepos::Skip => Private::Skip,
            PrivateRepos::Minimal => Private::Minimal,
        },
        ..Scope::for_user(&user)
    };
    let mut repos: Vec<Repo> = plan(&scope, &facts).into_iter().map(Repo::from).collect();
    if readme {
        for failed in client.readmes(&mut repos).await {
            eprintln!("README skipped: {failed}");
        }
    }
    // GitHub answered: only now is the old brain worth replacing
    if overwrite {
        let wiped = reset::wipe(&ctx.db, &ctx.config.database.uri).await?;
        if out.json {
            eprintln!("{}", serde_json::to_string(&wiped).unwrap_or_default());
        } else {
            println!("{}", reset::wiped_line(&wiped));
        }
    }
    // a dry run does all the writing in a transaction, then rolls it back
    let txn = ctx.db.begin().await?;
    let done = github::store(
        &txn,
        repos,
        &Filing {
            area_id: area,
            tiering: Some(Tiering {
                cold_after_months: cold_after,
                index_forks: forks == Forks::Index,
                now: chrono::Utc::now(),
            }),
        },
    )
    .await
    .map_err(explain)?;
    if dry_run {
        txn.rollback().await?;
        if !out.json {
            println!("dry run, nothing written:");
        }
    } else {
        txn.commit().await?;
    }
    out.many(&done, indexed_line);
    Ok(())
}
