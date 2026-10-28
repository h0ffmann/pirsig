use std::path::{Path, PathBuf};

use loco_rs::{app::AppContext, Error, Result};
use serde::Serialize;

use super::Output;
use crate::figures::{archify, graphify, Brain, Scope};

/// `psg figure -o para.tree.json`: the PARA tree as archify JSON IR.
///
/// Render it with `node <archify>/bin/archify.mjs finalize tree para.tree.json para.html`.
#[derive(Debug, clap::Args)]
pub struct FigureArgs {
    /// Write the IR here instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Figure title.
    #[arg(short, long, default_value = "Second brain")]
    title: String,
    /// HTML path archify writes (`meta.output`); defaults to the output file name with `.html`.
    #[arg(long, value_name = "FILE")]
    html: Option<String>,
    #[command(flatten)]
    scope: ScopeArgs,
}

/// `psg graph -o graphify-out/graph.json`: the brain as a graphify knowledge graph.
#[derive(Debug, clap::Args)]
pub struct GraphArgs {
    /// Write graph.json here instead of stdout (graphify looks in `graphify-out/graph.json`).
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    #[command(flatten)]
    scope: ScopeArgs,
}

#[derive(Debug, clap::Args)]
struct ScopeArgs {
    /// Include archived areas, projects and resources.
    #[arg(long)]
    archived: bool,
    /// Also include done tasks.
    #[arg(long)]
    all: bool,
}

impl From<ScopeArgs> for Scope {
    fn from(s: ScopeArgs) -> Self {
        Self {
            archived: s.archived,
            done: s.all,
        }
    }
}

pub async fn run_figure(ctx: &AppContext, out: &Output, args: FigureArgs) -> Result<()> {
    let brain = Brain::load(&ctx.db, args.scope.into()).await?;
    let html = args.html.unwrap_or_else(|| {
        args.output
            .as_deref()
            .map_or_else(|| "para.html".to_string(), html_name)
    });
    let opts = archify::Options {
        title: args.title,
        output: html,
    };
    emit(out, args.output.as_deref(), &archify::tree(&brain, &opts))
}

pub async fn run_graph(ctx: &AppContext, out: &Output, args: GraphArgs) -> Result<()> {
    let brain = Brain::load(&ctx.db, args.scope.into()).await?;
    emit(out, args.output.as_deref(), &graphify::graph(&brain))
}

/// `docs/archify/para.tree.json` -> `para.html`. archify wants a portable
/// relative path here; the real destination is `finalize`'s output argument.
fn html_name(path: &Path) -> String {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("para");
    let stem = name
        .strip_suffix(".tree.json")
        .or_else(|| name.strip_suffix(".json"))
        .unwrap_or(name);
    format!("{stem}.html")
}

/// The document always is JSON; `--json` only changes the confirmation line.
fn emit<T: Serialize>(out: &Output, path: Option<&Path>, doc: &T) -> Result<()> {
    let body = serde_json::to_string_pretty(doc).map_err(|e| Error::Message(e.to_string()))?;
    let Some(path) = path else {
        println!("{body}");
        return Ok(());
    };
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, body + "\n")?;
    let wrote = path.display().to_string();
    out.one(&serde_json::json!({ "wrote": wrote }), |_| {
        format!("wrote {wrote}")
    });
    Ok(())
}
