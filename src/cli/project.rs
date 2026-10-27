use clap::ArgGroup;
use loco_rs::{app::AppContext, Error, Result};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use super::{archived_tag, named, required, Output};
use crate::{
    dtos::projects::{CreateProject, ProjectDto, UpdateProject},
    models::{projects, tasks},
};

/// `ztk project -c -n "Run a marathon" -a 1`, `ztk project -u 2 --status done`, `ztk project -d 2`.
#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").args(["create", "update", "delete", "show", "archive", "unarchive"])))]
pub struct Args {
    /// Create a project (needs --name and --area).
    #[arg(short, long)]
    create: bool,
    /// Update the project with this id.
    #[arg(short, long, value_name = "ID")]
    update: Option<i64>,
    /// Delete the project with this id (and its tasks).
    #[arg(short, long, value_name = "ID")]
    delete: Option<i64>,
    /// Show one project.
    #[arg(short, long, value_name = "ID")]
    show: Option<i64>,
    /// Move the project to the archive.
    #[arg(long, value_name = "ID")]
    archive: Option<i64>,
    /// Bring the project back from the archive.
    #[arg(long, value_name = "ID")]
    unarchive: Option<i64>,

    /// Project name.
    #[arg(short, long)]
    name: Option<String>,
    /// Longer description.
    #[arg(long)]
    description: Option<String>,
    /// Free-form status, e.g. active, waiting, done.
    #[arg(long)]
    status: Option<String>,
    /// Area id: the project's parent, or the list filter.
    #[arg(short, long, value_name = "AREA_ID")]
    area: Option<i64>,
    /// When listing, show archived projects instead.
    #[arg(long)]
    archived: bool,
    /// Delete even when the project still has tasks.
    #[arg(short, long)]
    force: bool,
}

fn line(p: &ProjectDto) -> String {
    let status = p
        .status
        .as_deref()
        .map(|s| format!(" ({s})"))
        .unwrap_or_default();
    format!(
        "#{} {}{status} [area #{}]{}",
        p.id,
        p.name,
        p.area_id,
        archived_tag(p.archived_at.is_some())
    )
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    if args.create {
        let area_id = required(args.area, "--area")?;
        let params = CreateProject {
            name: required(args.name, "--name")?,
            description: args.description,
            status: args.status,
            area_id,
        };
        let item = named(
            projects::ActiveModel::create(db, params).await,
            "area",
            area_id,
        )?;
        out.one(&ProjectDto::from(item), line);
    } else if let Some(id) = args.update {
        let params = UpdateProject {
            name: args.name,
            description: args.description,
            status: args.status,
            area_id: args.area,
        };
        let item = named(projects::Model::find(db, id).await, "project", id)?;
        let item = named(
            item.update_with(db, params).await,
            "area",
            args.area.unwrap_or_default(),
        )?;
        out.one(&ProjectDto::from(item), line);
    } else if let Some(id) = args.delete {
        let item = named(projects::Model::find(db, id).await, "project", id)?;
        let children = tasks::Entity::find()
            .filter(tasks::Column::ProjectId.eq(id))
            .count(db)
            .await?;
        if children > 0 && !args.force {
            return Err(Error::Message(format!(
                "project #{id} has {children} task(s); archive it, or pass --force to delete them too"
            )));
        }
        item.remove(db).await?;
        out.deleted("project", id);
    } else if let Some(id) = args.show {
        out.one(
            &ProjectDto::from(named(projects::Model::find(db, id).await, "project", id)?),
            line,
        );
    } else if let Some(id) = args.archive.or(args.unarchive) {
        let item = named(projects::Model::find(db, id).await, "project", id)?;
        let item = item.set_archived(db, args.archive.is_some()).await?;
        out.one(&ProjectDto::from(item), line);
    } else {
        let items = projects::Entity::listing(args.area, args.archived)
            .all(db)
            .await?;
        let items: Vec<ProjectDto> = items.into_iter().map(Into::into).collect();
        out.many(&items, line);
    }
    Ok(())
}
