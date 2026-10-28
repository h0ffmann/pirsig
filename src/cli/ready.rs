use std::fmt::Write;

use loco_rs::{app::AppContext, Result};

use super::Output;
use crate::readiness::{self, Readiness, Status};

/// `psg ready`: how far the brain follows PARA and the Zettelkasten method,
/// and whether its Obsidian, RSS and arXiv channels are in use.
#[derive(Debug, clap::Args)]
pub struct Args {}

const fn mark(s: Status) -> &'static str {
    match s {
        Status::Ready => "✓",
        Status::Partial => "~",
        Status::Missing => "✗",
    }
}

fn render(r: &Readiness) -> String {
    let mut s = format!(
        "Methodology readiness: {}/{} checks ready\n",
        r.ready, r.total
    );
    for sec in &r.sections {
        let _ = write!(s, "\n{} {}\n", mark(sec.status), sec.name);
        for c in &sec.checks {
            let _ = writeln!(s, "  {} {:<40} {}", mark(c.status), c.name, c.detail);
            if let Some(fix) = &c.fix {
                let _ = writeln!(s, "    → {fix}");
            }
        }
    }
    s.trim_end().to_string()
}

pub async fn run(ctx: &AppContext, out: &Output, _args: Args) -> Result<()> {
    out.one(&readiness::report(&ctx.db).await?, render);
    Ok(())
}
