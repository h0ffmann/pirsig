//! `ztk`: manage the second brain from the terminal.
//!
//! The CLI boots the same Loco [`AppContext`] as the server and calls the
//! model layer directly, so the REST API and the CLI share one set of rules.
//! Each command lives in its own module; to add one, add a variant to
//! [`Command`] and a `run` function next to the others.
#![allow(clippy::missing_errors_doc)]

use clap::{Parser, Subcommand};
use loco_rs::{
    app::{AppContext, Hooks},
    boot::create_context,
    environment::{resolve_from_env, Environment},
    model::ModelError,
    Error, Result,
};
use migration::Migrator;
use serde::Serialize;

use crate::{app::App, dtos::common::Patch};

pub mod area;
pub mod arxiv;
pub mod bootstrap;
pub mod figure;
pub mod hippo;
pub mod link;
pub mod note;
pub mod obsidian;
pub mod para;
pub mod plast;
pub mod project;
pub mod ready;
pub mod resource;
pub mod rss;
pub mod slipbox;
pub mod task;

#[derive(Debug, Parser)]
#[command(name = "ztk", version, about = "Your second brain, in the terminal")]
pub struct Cli {
    /// Print results as JSON instead of text.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Areas: ongoing responsibilities (health, finances, ...).
    Area(area::Args),
    /// Projects: outcomes with an end, inside an area.
    Project(project::Args),
    /// Tasks: next actions inside a project.
    Task(task::Args),
    /// Resources: reference material, optionally tied to an area or project.
    Resource(resource::Args),
    /// Overview of everything in PARA, as a tree.
    Para(para::Args),
    /// Export the PARA tree as an archify diagram (JSON IR).
    Figure(figure::FigureArgs),
    /// Export the brain as a graphify knowledge graph (graph.json).
    Graph(figure::GraphArgs),
    /// Notes in the slipbox: capture, edit, search, show with backlinks.
    Note(note::Args),
    /// Link two notes and say why.
    Link(link::Args),
    /// The slipbox as a whole: inbox, orphans, tree, random, graph, stats, export.
    #[command(name = "box", alias = "slipbox")]
    Slipbox(slipbox::Args),
    /// Obsidian vault out and in (ztk.pkm.obsidian).
    Obsidian(obsidian::Args),
    /// RSS/Atom feeds in as resources, and the slipbox out as a feed (ztk.web.rss).
    Rss(rss::Args),
    /// arXiv papers in as resources, with citekeys and filing (ztk.sci.arxiv).
    Arxiv(arxiv::Args),
    /// Seed the brain from GitHub: a resource and a note per repo, indexed by category.
    Bootstrap(bootstrap::Args),
    /// Methodology readiness: PARA, Zettelkasten, Obsidian, RSS and arXiv checks.
    Ready(ready::Args),
    /// Hippo identities: a rhizome of notes for ADHD, autistic, neurotic and obsessive minds.
    Hippo(hippo::Args),
    /// Plasticity: quick exercises in math, abstraction, logic, music, literature, languages.
    Plast(plast::Args),
}

/// Run a parsed command against the database of the current environment
/// (`LOCO_ENV`, default `development`; `DATABASE_URL` overrides the db).
///
/// # Errors
/// When the app cannot boot or the command fails.
pub async fn run(cli: Cli) -> Result<()> {
    let ctx = boot().await?;
    let out = Output { json: cli.json };
    match cli.command {
        Command::Area(args) => area::run(&ctx, &out, args).await,
        Command::Project(args) => project::run(&ctx, &out, args).await,
        Command::Task(args) => task::run(&ctx, &out, args).await,
        Command::Resource(args) => resource::run(&ctx, &out, args).await,
        Command::Para(args) => para::run(&ctx, &out, args).await,
        Command::Figure(args) => figure::run_figure(&ctx, &out, args).await,
        Command::Graph(args) => figure::run_graph(&ctx, &out, args).await,
        Command::Note(args) => note::run(&ctx, &out, args).await,
        Command::Link(args) => link::run(&ctx, &out, args).await,
        Command::Slipbox(args) => slipbox::run(&ctx, &out, args).await,
        Command::Obsidian(args) => obsidian::run(&ctx, &out, args).await,
        Command::Rss(args) => rss::run(&ctx, &out, args).await,
        Command::Arxiv(args) => arxiv::run(&ctx, &out, args).await,
        Command::Bootstrap(args) => bootstrap::run(&ctx, &out, args).await,
        Command::Ready(args) => ready::run(&ctx, &out, args).await,
        Command::Hippo(args) => hippo::run(&ctx, &out, args).await,
        Command::Plast(args) => plast::run(&ctx, &out, args).await,
    }
}

async fn boot() -> Result<AppContext> {
    let environment: Environment = resolve_from_env().into();
    let config = App::load_config(&environment).await?;
    let ctx = create_context::<App>(&environment, config).await?;
    loco_rs::db::converge::<App, Migrator>(&ctx, &ctx.config.database).await?;
    Ok(ctx)
}

/// Where command results go: human-readable lines, or JSON for scripts.
pub struct Output {
    pub json: bool,
}

impl Output {
    pub fn one<T: Serialize>(&self, item: &T, line: impl FnOnce(&T) -> String) {
        if self.json {
            println!("{}", serde_json::to_string_pretty(item).unwrap_or_default());
        } else {
            println!("{}", line(item));
        }
    }

    pub fn many<T: Serialize>(&self, items: &[T], line: impl Fn(&T) -> String) {
        if self.json {
            println!(
                "{}",
                serde_json::to_string_pretty(items).unwrap_or_default()
            );
        } else if items.is_empty() {
            println!("(nothing here)");
        } else {
            for item in items {
                println!("{}", line(item));
            }
        }
    }

    pub fn deleted(&self, kind: &str, id: i64) {
        if self.json {
            println!("{}", serde_json::json!({ "deleted": kind, "id": id }));
        } else {
            println!("deleted {kind} #{id}");
        }
    }
}

/// Turn the model layer's bare "not found" into a message that names the thing,
/// and validation errors into `field: reason` lines.
pub(crate) fn named<T>(res: std::result::Result<T, ModelError>, what: &str, id: i64) -> Result<T> {
    res.map_err(|err| match err {
        ModelError::EntityNotFound => Error::Message(format!("{what} #{id} not found")),
        err => explain(err),
    })
}

/// A model error as people read it: validation errors become `field: reason` lines.
pub(crate) fn explain(err: ModelError) -> Error {
    match err {
        ModelError::Validation(v) => Error::Message(
            v.errors
                .iter()
                .flat_map(|(field, errs)| {
                    errs.iter().map(move |e| {
                        format!("{field}: {}", e.message.as_deref().unwrap_or(&e.code))
                    })
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        err => err.into(),
    }
}

/// `--due DATE` sets it, `--no-due` clears it, neither leaves it alone.
pub(crate) fn patch<T>(value: Option<T>, clear: bool) -> Patch<T> {
    if clear {
        Patch::Clear
    } else {
        value.into()
    }
}

pub(crate) fn required<T>(value: Option<T>, flag: &str) -> Result<T> {
    value.ok_or_else(|| Error::Message(format!("{flag} is required")))
}

pub(crate) const fn archived_tag(archived: bool) -> &'static str {
    if archived {
        " [archived]"
    } else {
        ""
    }
}
