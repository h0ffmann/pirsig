use std::path::PathBuf;

use clap::Subcommand;
use loco_rs::{app::AppContext, Error, Result};
use sea_orm::EntityTrait;

use super::{explain, named, Output};
use crate::{dtos::feeds::FeedDto, ingest::rss, models::feeds};

/// `psg rss add URL -a 2`, `psg rss pull`, `psg rss out -o notes.xml`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Subscribe to a feed; its entries will be filed under the area or project.
    Add {
        url: String,
        /// Area id to file entries under.
        #[arg(short, long, value_name = "AREA_ID")]
        area: Option<i64>,
        /// Project id to file entries under.
        #[arg(short, long, value_name = "PROJECT_ID")]
        project: Option<i64>,
    },
    /// List the feeds (the default).
    List,
    /// Fetch one feed, or all of them, and file new entries as resources.
    Pull {
        #[arg(value_name = "FEED_ID")]
        id: Option<i64>,
    },
    /// Unsubscribe; the resources it brought stay.
    Remove {
        #[arg(value_name = "FEED_ID")]
        id: i64,
    },
    /// The slipbox out: notes as an RSS 2.0 feed.
    Out {
        /// Write to this file instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Note kind: permanent (default), literature or hub. Fleeting never goes out.
        #[arg(short, long)]
        kind: Option<String>,
        /// Only notes with this tag.
        #[arg(long)]
        tag: Option<String>,
        /// How many notes, newest first.
        #[arg(short = 'n', long, default_value_t = 50)]
        limit: u64,
        /// Where the brain is served, for item links.
        #[arg(long, default_value = "http://localhost:5150")]
        link: String,
        /// Feed title.
        #[arg(long, default_value = "Pirsig slipbox")]
        title: String,
    },
}

fn line(f: &FeedDto) -> String {
    let title = f.title.as_deref().unwrap_or("(not pulled yet)");
    let filed = match (f.area_id, f.project_id) {
        (_, Some(p)) => format!(" [project #{p}]"),
        (Some(a), None) => format!(" [area #{a}]"),
        _ => String::new(),
    };
    format!("#{} {title} <{}>{filed}", f.id, f.url)
}

fn pulled(r: &rss::PullReport) -> String {
    format!(
        "#{} {}: {} new, {} updated",
        r.feed_id,
        r.title.as_deref().unwrap_or("?"),
        r.new,
        r.updated
    )
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    match args.action.unwrap_or(Action::List) {
        Action::Add { url, area, project } => {
            let feed = feeds::ActiveModel::subscribe(db, &url, area, project)
                .await
                .map_err(explain)?;
            out.one(&FeedDto::from(feed), line);
        }
        Action::List => {
            let all: Vec<FeedDto> = feeds::Entity::listing()
                .all(db)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();
            out.many(&all, line);
        }
        Action::Pull { id } => {
            let targets = match id {
                Some(id) => vec![named(feeds::Model::find(db, id).await, "feed", id)?],
                None => feeds::Entity::find().all(db).await?,
            };
            let mut reports = Vec::new();
            for feed in targets {
                let url = feed.url.clone();
                match rss::pull(db, feed).await {
                    Ok(r) => reports.push(r),
                    // one broken feed does not stop the others
                    Err(e) => eprintln!("{url}: {e}"),
                }
            }
            out.many(&reports, pulled);
        }
        Action::Remove { id } => {
            named(feeds::Model::find(db, id).await, "feed", id)?
                .remove(db)
                .await?;
            out.deleted("feed", id);
        }
        Action::Out {
            output,
            kind,
            tag,
            limit,
            link,
            title,
        } => {
            let items = rss::feed_notes(db, kind.as_deref(), tag.as_deref(), limit)
                .await
                .map_err(explain)?;
            let xml = rss::render(&title, &link, &items);
            match output {
                Some(path) => {
                    std::fs::write(&path, xml)
                        .map_err(|e| Error::Message(format!("{}: {e}", path.display())))?;
                    eprintln!("wrote {} notes to {}", items.len(), path.display());
                }
                None => print!("{xml}"),
            }
        }
    }
    Ok(())
}
