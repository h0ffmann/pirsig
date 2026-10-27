use clap::ArgGroup;
use loco_rs::{app::AppContext, Error, Result};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use super::{archived_tag, named, required, Output};
use crate::{
    dtos::areas::{AreaDto, CreateArea, UpdateArea},
    models::{areas, projects},
};

/// `ztk area -c -n Health`, `ztk area -u 1 -n Fitness`, `ztk area -d 1`, `ztk area` (list).
#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").args(["create", "update", "delete", "show", "archive", "unarchive"])))]
pub struct Args {
    /// Create an area (needs --name).
    #[arg(short, long)]
    create: bool,
    /// Update the area with this id.
    #[arg(short, long, value_name = "ID")]
    update: Option<i64>,
    /// Delete the area with this id (and its projects and tasks).
    #[arg(short, long, value_name = "ID")]
    delete: Option<i64>,
    /// Show one area.
    #[arg(short, long, value_name = "ID")]
    show: Option<i64>,
    /// Move the area to the archive.
    #[arg(long, value_name = "ID")]
    archive: Option<i64>,
    /// Bring the area back from the archive.
    #[arg(long, value_name = "ID")]
    unarchive: Option<i64>,

    /// Area name.
    #[arg(short, long)]
    name: Option<String>,
    /// Longer description.
    #[arg(long)]
    description: Option<String>,
    /// When listing, show archived areas instead.
    #[arg(long)]
    archived: bool,
    /// Delete even when the area still has projects.
    #[arg(short, long)]
    force: bool,
}

fn line(a: &AreaDto) -> String {
    let desc = a
        .description
        .as_deref()
        .map(|d| format!(" - {d}"))
        .unwrap_or_default();
    format!(
        "#{} {}{desc}{}",
        a.id,
        a.name,
        archived_tag(a.archived_at.is_some())
    )
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    if args.create {
        let params = CreateArea {
            name: required(args.name, "--name")?,
            description: args.description,
        };
        let item = areas::ActiveModel::create(db, params).await?;
        out.one(&AreaDto::from(item), line);
    } else if let Some(id) = args.update {
        let params = UpdateArea {
            name: args.name,
            description: args.description,
        };
        let item = named(areas::Model::find(db, id).await, "area", id)?;
        out.one(&AreaDto::from(item.update_with(db, params).await?), line);
    } else if let Some(id) = args.delete {
        let item = named(areas::Model::find(db, id).await, "area", id)?;
        let children = projects::Entity::find()
            .filter(projects::Column::AreaId.eq(id))
            .count(db)
            .await?;
        if children > 0 && !args.force {
            return Err(Error::Message(format!(
                "area #{id} has {children} project(s); archive it, or pass --force to delete them too"
            )));
        }
        item.remove(db).await?;
        out.deleted("area", id);
    } else if let Some(id) = args.show {
        out.one(
            &AreaDto::from(named(areas::Model::find(db, id).await, "area", id)?),
            line,
        );
    } else if let Some(id) = args.archive.or(args.unarchive) {
        let item = named(areas::Model::find(db, id).await, "area", id)?;
        let item = item.set_archived(db, args.archive.is_some()).await?;
        out.one(&AreaDto::from(item), line);
    } else {
        let items = areas::Entity::listing(args.archived).all(db).await?;
        let items: Vec<AreaDto> = items.into_iter().map(Into::into).collect();
        out.many(&items, line);
    }
    Ok(())
}
