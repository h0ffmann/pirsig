use loco_rs::{app::AppContext, Result};

use super::{
    note::{detail, found},
    Output,
};
use crate::{dtos::notes::NoteDetail, models::notes};

/// `psg link 21a 3 -w "contradicts the claim in 3"`, `psg link -d 21a 3`.
///
/// Links written as `[[3]]` in a body appear on their own; use this for links
/// that deserve a reason.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// The note the link starts from.
    #[arg(value_name = "FROM")]
    from: String,
    /// The note it points to.
    #[arg(value_name = "TO")]
    to: String,
    /// Why the two belong together.
    #[arg(short, long)]
    why: Option<String>,
    /// Remove the link instead.
    #[arg(short, long)]
    delete: bool,
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    let from = found(notes::Model::find(db, &args.from).await, &args.from)?;
    let to = found(notes::Model::find(db, &args.to).await, &args.to)?;
    if args.delete {
        if !from.unlink(db, &to).await? {
            return Err(loco_rs::Error::Message(format!(
                "note {} does not link to {}",
                args.from, args.to
            )));
        }
    } else {
        from.link_to(db, &to, args.why).await?;
    }
    out.one(&NoteDetail::load(db, from).await?, detail);
    Ok(())
}
