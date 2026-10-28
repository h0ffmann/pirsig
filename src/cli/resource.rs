use clap::ArgGroup;
use loco_rs::{app::AppContext, Result};

use super::{archived_tag, named, patch, required, Output};
use crate::{
    dtos::resources::{CreateResource, ResourceDto, UpdateResource},
    models::resources,
};

/// `psg resource -c -t "Rust book" --url https://doc.rust-lang.org/book/ -a 1`, `psg resource -d 3`.
// clap flags are bools by nature
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").args(["create", "update", "delete", "show", "archive", "unarchive"])))]
pub struct Args {
    /// Create a resource (needs --title).
    #[arg(short, long)]
    create: bool,
    /// Update the resource with this id.
    #[arg(short, long, value_name = "ID")]
    update: Option<i64>,
    /// Delete the resource with this id.
    #[arg(short, long, value_name = "ID")]
    delete: Option<i64>,
    /// Show one resource.
    #[arg(short, long, value_name = "ID")]
    show: Option<i64>,
    /// Move the resource to the archive.
    #[arg(long, value_name = "ID")]
    archive: Option<i64>,
    /// Bring the resource back from the archive.
    #[arg(long, value_name = "ID")]
    unarchive: Option<i64>,

    /// Resource title.
    #[arg(short, long)]
    title: Option<String>,
    /// Link to the source (http or https); `""` clears it.
    #[arg(long)]
    url: Option<String>,
    /// Free-form body (e.g. Markdown); `""` clears it.
    #[arg(long)]
    body: Option<String>,
    /// article, book, paper, repo, video, web or other; `""` clears it.
    #[arg(short, long)]
    kind: Option<String>,
    /// BibTeX key that literature notes cite (`vaswani2017attention`); `""` clears it.
    #[arg(long)]
    citekey: Option<String>,
    /// Area id: link the resource to an area, or the list filter.
    #[arg(short, long, value_name = "AREA_ID")]
    area: Option<i64>,
    /// Unfile the resource from its area.
    #[arg(long, conflicts_with = "area")]
    no_area: bool,
    /// Project id: link the resource to a project, or the list filter.
    #[arg(short, long, value_name = "PROJECT_ID")]
    project: Option<i64>,
    /// Unfile the resource from its project.
    #[arg(long, conflicts_with = "project")]
    no_project: bool,
    /// When listing, show archived resources instead.
    #[arg(long)]
    archived: bool,
}

fn line(r: &ResourceDto) -> String {
    let url = r
        .url
        .as_deref()
        .map(|u| format!(" <{u}>"))
        .unwrap_or_default();
    let cite = r
        .citekey
        .as_deref()
        .map(|k| format!(" @{k}"))
        .unwrap_or_default();
    format!(
        "#{} {}{cite}{url}{}",
        r.id,
        r.title,
        archived_tag(r.archived_at.is_some())
    )
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    if args.create {
        let params = CreateResource {
            title: required(args.title, "--title")?,
            url: args.url,
            body: args.body,
            area_id: args.area,
            project_id: args.project,
            kind: args.kind,
            citekey: args.citekey,
            external_id: None,
        };
        let item = named(
            resources::ActiveModel::create(db, params).await,
            "area or project",
            args.area.or(args.project).unwrap_or_default(),
        )?;
        out.one(&ResourceDto::from(item), line);
    } else if let Some(id) = args.update {
        let params = UpdateResource {
            title: args.title,
            url: args.url.into(),
            body: args.body.into(),
            area_id: patch(args.area, args.no_area),
            project_id: patch(args.project, args.no_project),
            kind: args.kind.into(),
            citekey: args.citekey.into(),
        };
        let item = named(resources::Model::find(db, id).await, "resource", id)?;
        let item = named(
            item.update_with(db, params).await,
            "area or project",
            args.area.or(args.project).unwrap_or_default(),
        )?;
        out.one(&ResourceDto::from(item), line);
    } else if let Some(id) = args.delete {
        named(resources::Model::find(db, id).await, "resource", id)?
            .remove(db)
            .await?;
        out.deleted("resource", id);
    } else if let Some(id) = args.show {
        out.one(
            &ResourceDto::from(named(resources::Model::find(db, id).await, "resource", id)?),
            line,
        );
    } else if let Some(id) = args.archive.or(args.unarchive) {
        let item = named(resources::Model::find(db, id).await, "resource", id)?;
        let item = item.set_archived(db, args.archive.is_some()).await?;
        out.one(&ResourceDto::from(item), line);
    } else {
        let items = resources::Entity::listing(args.area, args.project, args.archived)
            .all(db)
            .await?;
        let items: Vec<ResourceDto> = items.into_iter().map(Into::into).collect();
        out.many(&items, line);
    }
    Ok(())
}
