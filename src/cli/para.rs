use std::fmt::Write;

use loco_rs::{app::AppContext, Result};
use serde::Serialize;

use super::Output;
use crate::{
    dtos::{areas::AreaDto, projects::ProjectDto, resources::ResourceDto, tasks::TaskDto},
    models::{areas, projects, resources, tasks},
};

/// `ztk para`: areas, their projects and open tasks, resources, and the archive size.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Also list done tasks.
    #[arg(long)]
    all: bool,
}

#[derive(Serialize)]
struct Overview {
    areas: Vec<AreaNode>,
    resources: Vec<ResourceDto>,
    archived: Archived,
}

#[derive(Serialize)]
struct AreaNode {
    #[serde(flatten)]
    area: AreaDto,
    projects: Vec<ProjectNode>,
}

#[derive(Serialize)]
struct ProjectNode {
    #[serde(flatten)]
    project: ProjectDto,
    tasks: Vec<TaskDto>,
}

#[derive(Serialize)]
struct Archived {
    areas: usize,
    projects: usize,
    resources: usize,
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    let all_projects = projects::Entity::listing(None, false).all(db).await?;
    let all_tasks = tasks::Entity::listing(None, (!args.all).then_some(false))
        .all(db)
        .await?;

    let areas = areas::Entity::listing(false)
        .all(db)
        .await?
        .into_iter()
        .map(|area| AreaNode {
            projects: all_projects
                .iter()
                .filter(|p| p.area_id == area.id)
                .map(|p| ProjectNode {
                    tasks: all_tasks
                        .iter()
                        .filter(|t| t.project_id == p.id)
                        .cloned()
                        .map(Into::into)
                        .collect(),
                    project: p.clone().into(),
                })
                .collect(),
            area: area.into(),
        })
        .collect();

    let overview = Overview {
        areas,
        resources: resources::Entity::listing(None, None, false)
            .all(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
        archived: Archived {
            areas: areas::Entity::listing(true).all(db).await?.len(),
            projects: projects::Entity::listing(None, true).all(db).await?.len(),
            resources: resources::Entity::listing(None, None, true)
                .all(db)
                .await?
                .len(),
        },
    };

    out.one(&overview, render);
    Ok(())
}

fn render(o: &Overview) -> String {
    let mut s = String::from("Areas\n");
    for a in &o.areas {
        let _ = writeln!(s, "  #{} {}", a.area.id, a.area.name);
        for p in &a.projects {
            let status = p
                .project
                .status
                .as_deref()
                .map(|st| format!(" ({st})"))
                .unwrap_or_default();
            let _ = writeln!(s, "    #{} {}{status}", p.project.id, p.project.name);
            for t in &p.tasks {
                let check = if t.done { "x" } else { " " };
                let _ = writeln!(s, "      [{check}] #{} {}", t.id, t.title);
            }
        }
    }
    s += "Resources\n";
    for r in &o.resources {
        let _ = writeln!(s, "  #{} {}", r.id, r.title);
    }
    let _ = write!(
        s,
        "Archive: {} area(s), {} project(s), {} resource(s)",
        o.archived.areas, o.archived.projects, o.archived.resources
    );
    s
}
