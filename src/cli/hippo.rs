use std::fmt::Write;

use clap::ArgGroup;
use loco_rs::{app::AppContext, Result};

use super::{explain, named, required, Output};
use crate::{
    dtos::hippos::{CreateHippo, HippoDto, RhizomeMap, UpdateHippo, WeaveReport},
    models::hippos,
};

/// `psg hippo -c -n me --trait adhd --fixation "rogue waves"`, `psg hippo --map 1`,
/// `psg hippo --weave 1`, then `psg note -c "an idea" -H 1`.
#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").args(["create", "update", "delete", "show", "map", "weave"])))]
pub struct Args {
    /// Create a Hippo identity (needs --name).
    #[arg(short, long)]
    create: bool,
    /// Update the Hippo with this id.
    #[arg(short, long, value_name = "ID")]
    update: Option<i64>,
    /// Delete the Hippo; its notes stay in the slipbox with their links.
    #[arg(short, long, value_name = "ID")]
    delete: Option<i64>,
    /// Show one Hippo.
    #[arg(short, long, value_name = "ID")]
    show: Option<i64>,
    /// The rhizome: where you left off, the plateaus, the pieces.
    #[arg(short, long, value_name = "ID")]
    map: Option<i64>,
    /// Link every note with too few links, and bridge the pieces into one.
    #[arg(short, long, value_name = "ID")]
    weave: Option<i64>,

    /// Name (yours, or a handle).
    #[arg(short, long)]
    name: Option<String>,
    /// A sentence on how this mind works; `""` clears it.
    #[arg(long)]
    description: Option<String>,
    /// Self-described trait, repeatable: adhd, autistic, neurotic, obsessive.
    /// On update, replaces the list.
    #[arg(short, long = "trait", value_name = "TRAIT")]
    traits: Vec<String>,
    /// Obsession or special interest, repeatable. On update, replaces the list.
    #[arg(short, long = "fixation", value_name = "TOPIC")]
    fixations: Vec<String>,
}

fn line(h: &HippoDto) -> String {
    let traits = if h.traits.is_empty() {
        String::new()
    } else {
        format!(" ({})", h.traits.join(", "))
    };
    let fix = if h.fixations.is_empty() {
        String::new()
    } else {
        format!(" ~ {}", h.fixations.join(" · "))
    };
    format!("#{} {}{traits}{fix}", h.id, h.name)
}

fn map_text(m: &RhizomeMap) -> String {
    let mut s = format!(
        "{}: {} notes, {} links, {}\n",
        line(&m.hippo),
        m.notes,
        m.links,
        match m.components.len() {
            0 => "empty".to_string(),
            1 => "one rhizome".to_string(),
            n => format!("{n} pieces (psg hippo --weave {} joins them)", m.hippo.id),
        }
    );
    if !m.resume.is_empty() {
        s.push_str("\nWhere you left off\n");
        for n in &m.resume {
            let _ = writeln!(s, "  {} {}", n.zid, n.title);
        }
    }
    for p in &m.plateaus {
        let _ = writeln!(s, "\n~ {} ({})", p.fixation, p.notes.len());
        for n in &p.notes {
            let _ = writeln!(s, "  {} {}", n.zid, n.title);
        }
    }
    if m.components.len() > 1 {
        s.push_str("\nPieces\n");
        for (i, c) in m.components.iter().enumerate() {
            let names: Vec<&str> = c.iter().take(4).map(|n| n.zid.as_str()).collect();
            let _ = writeln!(s, "  {}. {} note(s): {}", i + 1, c.len(), names.join(", "));
        }
    }
    s.trim_end().to_string()
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    let traits = (!args.traits.is_empty()).then_some(args.traits);
    let fixations = (!args.fixations.is_empty()).then_some(args.fixations);
    if args.create {
        let params = CreateHippo {
            name: required(args.name, "--name")?,
            description: args.description,
            traits: traits.unwrap_or_default(),
            fixations: fixations.unwrap_or_default(),
        };
        let h = hippos::ActiveModel::create(db, params)
            .await
            .map_err(explain)?;
        out.one(&HippoDto::from(h), line);
    } else if let Some(id) = args.update {
        let params = UpdateHippo {
            name: args.name,
            description: args.description.into(),
            traits,
            fixations,
        };
        let h = named(hippos::Model::find(db, id).await, "hippo", id)?;
        let h = named(h.update_with(db, params).await, "hippo", id)?;
        out.one(&HippoDto::from(h), line);
    } else if let Some(id) = args.delete {
        named(hippos::Model::find(db, id).await, "hippo", id)?
            .remove(db)
            .await?;
        out.deleted("hippo", id);
    } else if let Some(id) = args.show {
        let h = named(hippos::Model::find(db, id).await, "hippo", id)?;
        out.one(&HippoDto::from(h), line);
    } else if let Some(id) = args.map {
        let h = named(hippos::Model::find(db, id).await, "hippo", id)?;
        out.one(&h.map(db).await?, map_text);
    } else if let Some(id) = args.weave {
        let h = named(hippos::Model::find(db, id).await, "hippo", id)?;
        let r = h.weave_all(db).await?;
        out.one(&r, |r: &WeaveReport| {
            format!("{} link(s) woven, {} piece(s) bridged", r.woven, r.bridged)
        });
    } else {
        let all: Vec<HippoDto> = hippos::Entity::listing()
            .all(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        out.many(&all, line);
    }
    Ok(())
}
