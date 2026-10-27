use std::path::PathBuf;

use clap::Subcommand;
use loco_rs::{app::AppContext, Error, Result};

use super::{
    note::{detail, line},
    Output,
};
use crate::{
    dtos::notes::{GraphDto, NoteDetail, NoteDto, StatsDto, TreeEntry},
    models::notes,
};

/// `ztk box` (stats), `ztk box inbox`, `ztk box tree`, `ztk box random`, `ztk box export ./vault`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    view: Option<View>,
}

#[derive(Debug, Subcommand)]
enum View {
    /// Fleeting notes waiting to be processed into permanent ones.
    Inbox,
    /// Notes with no links in or out.
    Orphans,
    /// A random note, to stumble on old ideas.
    Random,
    /// Every note in Folgezettel order, indented by depth.
    Tree,
    /// Every note and link as JSON, for graph tools.
    Graph,
    /// Tags with how many notes use them.
    Tags,
    /// Counts: notes, links, inbox, orphans, kinds, tags.
    Stats,
    /// Write every note as Markdown (`<zid> <title>.md`) into a folder.
    Export {
        /// Destination folder; created if missing.
        dir: PathBuf,
    },
}

fn tree_line(e: &TreeEntry) -> String {
    format!("{}{} {}", "  ".repeat(e.depth), e.zid, e.title)
}

fn stats_line(s: &StatsDto) -> String {
    let kinds = s
        .by_kind
        .iter()
        .map(|k| format!("{} {}", k.count, k.name))
        .collect::<Vec<_>>()
        .join(", ");
    let tags = s
        .tags
        .iter()
        .take(10)
        .map(|t| format!("#{} ({})", t.name, t.count))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{} notes, {} links\n{} in inbox, {} orphans\nkinds: {}\ntop tags: {}",
        s.notes,
        s.links,
        s.inbox,
        s.orphans,
        if kinds.is_empty() { "-" } else { &kinds },
        if tags.is_empty() { "-" } else { &tags },
    )
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    match args.view.unwrap_or(View::Stats) {
        View::Inbox => {
            let found = notes::Entity::inbox().all(db).await?;
            out.many(&NoteDto::many(db, found).await?, line);
        }
        View::Orphans => {
            let found = notes::Entity::orphans().all(db).await?;
            out.many(&NoteDto::many(db, found).await?, line);
        }
        View::Random => match notes::Entity::random(db).await? {
            Some(note) => out.one(&NoteDetail::load(db, note).await?, detail),
            None => println!("(the slipbox is empty)"),
        },
        View::Tree => {
            let entries: Vec<TreeEntry> = notes::Entity::tree(db)
                .await?
                .iter()
                .map(TreeEntry::from)
                .collect();
            out.many(&entries, tree_line);
        }
        View::Graph => {
            let graph = GraphDto::load(db).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&graph).unwrap_or_default()
            );
        }
        View::Tags => {
            let tags: Vec<(String, i64)> = notes::Entity::tag_counts(db).await?;
            out.many(&tags, |(name, count)| format!("#{name} ({count})"));
        }
        View::Stats => out.one(&notes::Entity::stats(db).await?, stats_line),
        View::Export { dir } => {
            let io = |e: std::io::Error| Error::Message(format!("{}: {e}", dir.display()));
            std::fs::create_dir_all(&dir).map_err(io)?;
            let all = notes::Entity::tree(db).await?;
            for note in &all {
                let md = note.to_markdown(&note.tags(db).await?, &note.links(db).await?);
                std::fs::write(dir.join(file_name(note)), md).map_err(io)?;
            }
            if out.json {
                println!(
                    "{}",
                    serde_json::json!({ "exported": all.len(), "dir": dir })
                );
            } else {
                println!("exported {} notes to {}", all.len(), dir.display());
            }
        }
    }
    Ok(())
}

/// `21a Some title.md`, with characters that trouble file systems dropped.
fn file_name(note: &notes::Model) -> String {
    let title: String = note
        .title
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect();
    format!("{} {}.md", note.zid, title.trim())
}
