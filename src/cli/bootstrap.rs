use std::fmt::Write;

use clap::{Subcommand, ValueEnum};
use loco_rs::{app::AppContext, Result};
use sea_orm::TransactionTrait;
use ztk_github::{
    index::{plan, Depth},
    scope::{Private, Scope},
};

use super::{explain, Output};
use crate::ingest::github::{self, Client, Filing, Indexed, Repo};

/// `ztk bootstrap gh`, `ztk bootstrap gh --private minimal --readme --dry-run`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    source: Source,
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
    Gh {
        /// GitHub user (default: the token's user).
        user: Option<String>,
        /// Your private repositories (only seen with your own token).
        #[arg(long, value_enum, default_value = "skip")]
        private: PrivateRepos,
        /// Leave out your public forks.
        #[arg(long)]
        no_forks: bool,
        /// Also read each public repo's README for its first paragraph.
        #[arg(long)]
        readme: bool,
        /// File here the repos whose category has no area in the brain.
        #[arg(short, long, value_name = "AREA_ID")]
        area: Option<i64>,
        /// Show what would be indexed, and write nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

fn indexed_line(i: &Indexed) -> String {
    let what = if i.created { "added" } else { "known" };
    let mut s = format!("{what} #{} {} repo/{}", i.resource_id, i.repo, i.category);
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
    if let Some(z) = &i.note {
        let _ = write!(s, "; note {z}");
    }
    s
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let Source::Gh {
        user,
        private,
        no_forks,
        readme,
        area,
        dry_run,
    } = args.source;
    let client = Client::from_env()?;
    let (user, facts) = client.facts(user.as_deref()).await?;
    let scope = Scope {
        forks: !no_forks,
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
    // a dry run does all the writing in a transaction, then rolls it back
    let txn = ctx.db.begin().await?;
    let done = github::store(&txn, repos, &Filing { area_id: area })
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
