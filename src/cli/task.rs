use clap::ArgGroup;
use loco_rs::{app::AppContext, Result};
use sea_orm::prelude::Date;

use super::{named, required, Output};
use crate::{
    dtos::tasks::{CreateTask, TaskDto, UpdateTask},
    models::tasks,
};

/// `ztk task -c -t "Buy shoes" -p 2 --due 2026-11-01`, `ztk task --done 5`, `ztk task -d 5`.
#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").args(["create", "update", "delete", "show", "done", "undone"])))]
pub struct Args {
    /// Create a task (needs --title and --project).
    #[arg(short, long)]
    create: bool,
    /// Update the task with this id.
    #[arg(short, long, value_name = "ID")]
    update: Option<i64>,
    /// Delete the task with this id.
    #[arg(short, long, value_name = "ID")]
    delete: Option<i64>,
    /// Show one task.
    #[arg(short, long, value_name = "ID")]
    show: Option<i64>,
    /// Mark the task done.
    #[arg(long, value_name = "ID")]
    done: Option<i64>,
    /// Mark the task not done.
    #[arg(long, value_name = "ID")]
    undone: Option<i64>,

    /// Task title.
    #[arg(short, long)]
    title: Option<String>,
    /// Free-form notes.
    #[arg(long)]
    notes: Option<String>,
    /// Due date, YYYY-MM-DD.
    #[arg(long, value_name = "DATE")]
    due: Option<Date>,
    /// Project id: the task's parent, or the list filter.
    #[arg(short, long, value_name = "PROJECT_ID")]
    project: Option<i64>,
    /// When listing, show only open tasks.
    #[arg(short, long)]
    open: bool,
}

fn line(t: &TaskDto) -> String {
    let check = if t.done { "x" } else { " " };
    let due = t.due_on.map(|d| format!(" (due {d})")).unwrap_or_default();
    format!(
        "[{check}] #{} {}{due} [project #{}]",
        t.id, t.title, t.project_id
    )
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    if args.create {
        let project_id = required(args.project, "--project")?;
        let params = CreateTask {
            title: required(args.title, "--title")?,
            notes: args.notes,
            done: false,
            due_on: args.due,
            project_id,
        };
        let item = named(
            tasks::ActiveModel::create(db, params).await,
            "project",
            project_id,
        )?;
        out.one(&TaskDto::from(item), line);
    } else if let Some(id) = args.update {
        let params = UpdateTask {
            title: args.title,
            notes: args.notes,
            done: None,
            due_on: args.due,
            project_id: args.project,
        };
        let item = named(tasks::Model::find(db, id).await, "task", id)?;
        let item = named(
            item.update_with(db, params).await,
            "project",
            args.project.unwrap_or_default(),
        )?;
        out.one(&TaskDto::from(item), line);
    } else if let Some(id) = args.delete {
        named(tasks::Model::find(db, id).await, "task", id)?
            .remove(db)
            .await?;
        out.deleted("task", id);
    } else if let Some(id) = args.show {
        out.one(
            &TaskDto::from(named(tasks::Model::find(db, id).await, "task", id)?),
            line,
        );
    } else if let Some(id) = args.done.or(args.undone) {
        let params = UpdateTask {
            done: Some(args.done.is_some()),
            ..Default::default()
        };
        let item = named(tasks::Model::find(db, id).await, "task", id)?;
        out.one(&TaskDto::from(item.update_with(db, params).await?), line);
    } else {
        let done = args.open.then_some(false);
        let items = tasks::Entity::listing(args.project, done).all(db).await?;
        let items: Vec<TaskDto> = items.into_iter().map(Into::into).collect();
        out.many(&items, line);
    }
    Ok(())
}
