use std::fmt::Write;

use clap::Subcommand;
use loco_rs::{app::AppContext, Result};

use super::{explain, Output};
use crate::ingest::arxiv::{self, Filing, Ingested, Paper};

/// `ztk arxiv add 1706.03762 -a 3 --note`, `ztk arxiv search "rogue waves" -n 5`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Bring papers in as resources: ids, arXiv:ids, abs/ or pdf/ URLs.
    Add {
        #[arg(required = true, value_name = "ID")]
        ids: Vec<String>,
        /// File under this area (default: the one its categories point to).
        #[arg(short, long, value_name = "AREA_ID")]
        area: Option<i64>,
        /// File under this project.
        #[arg(short, long, value_name = "PROJECT_ID")]
        project: Option<i64>,
        /// Also put a fleeting "Read:" note in the inbox.
        #[arg(long)]
        note: bool,
    },
    /// Search arXiv; prints ids to add. Plain words, or arXiv syntax (ti:, au:, cat:).
    Search {
        query: String,
        #[arg(short = 'n', long, default_value_t = 10)]
        max: u32,
    },
}

fn paper_line(p: &Paper) -> String {
    let year = p
        .published
        .map(|d| d.format(" (%Y)").to_string())
        .unwrap_or_default();
    let who = match p.authors.as_slice() {
        [] => String::new(),
        [one] => format!(" — {one}"),
        [first, ..] => format!(" — {first} et al."),
    };
    format!("{:<16} {}{who}{year}", p.id, p.title)
}

fn ingested_line(i: &Ingested) -> String {
    let what = if i.created { "added" } else { "updated" };
    let key = i
        .citekey
        .as_deref()
        .map(|k| format!(" @{k}"))
        .unwrap_or_default();
    let mut s = format!("{what} #{} {}{key}", i.resource_id, i.title);
    match (&i.area, i.could_go_in.as_slice()) {
        (Some(a), _) => {
            let _ = write!(s, " [{a}]");
        }
        (None, []) => s.push_str(" [unfiled]"),
        (None, many) => {
            let _ = write!(s, " [unfiled: could go in {}]", many.join(" or "));
        }
    }
    if let Some(z) = &i.note {
        let _ = write!(s, "; inbox note {z}");
    }
    s
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    match args.action {
        Action::Add {
            ids,
            area,
            project,
            note,
        } => {
            let papers = arxiv::fetch_ids(&ids).await?;
            let filing = Filing {
                area_id: area,
                project_id: project,
                note,
            };
            let done = arxiv::store(&ctx.db, papers, &filing)
                .await
                .map_err(explain)?;
            out.many(&done, ingested_line);
        }
        Action::Search { query, max } => {
            let found = arxiv::search(&query, max).await?;
            out.many(&found, paper_line);
        }
    }
    Ok(())
}
