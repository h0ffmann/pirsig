//! Figures of the brain, as input for external renderers.
//!
//! Pirsig does not draw. It exports what is in the brain in the formats that
//! dedicated tools already render well:
//!
//! - [`archify`]: the PARA tree as archify's typed JSON IR (`diagram_type: "tree"`),
//!   which `archify finalize tree` turns into a validated standalone HTML figure.
//! - [`graphify`]: the brain as a graphify `graph.json` knowledge graph, which
//!   `graphify cluster-only`, `query`, `path`, `explain` and `export html` read.
//!
//! Both are pure functions of a [`Brain`] snapshot, shared by `psg figure`,
//! `psg graph` and the `/api/figures` endpoints.

use loco_rs::Result;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};

use crate::models::{areas, projects, resources, tasks};

pub mod archify;
pub mod graphify;

/// What goes into a figure.
#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
pub struct Scope {
    /// Include archived areas, projects and resources.
    #[serde(default)]
    pub archived: bool,
    /// Include done tasks.
    #[serde(default)]
    pub done: bool,
}

/// Everything in PARA at one point in time.
#[derive(Debug, Default, Clone)]
pub struct Brain {
    pub areas: Vec<areas::Model>,
    pub projects: Vec<projects::Model>,
    pub tasks: Vec<tasks::Model>,
    pub resources: Vec<resources::Model>,
}

impl Brain {
    /// # Errors
    /// When a query fails.
    pub async fn load(db: &DatabaseConnection, scope: Scope) -> Result<Self> {
        let (areas, projects, resources) = if scope.archived {
            (
                areas::Entity::find()
                    .order_by_asc(areas::Column::Id)
                    .all(db)
                    .await?,
                projects::Entity::find()
                    .order_by_asc(projects::Column::Id)
                    .all(db)
                    .await?,
                resources::Entity::find()
                    .order_by_asc(resources::Column::Id)
                    .all(db)
                    .await?,
            )
        } else {
            (
                areas::Entity::listing(false).all(db).await?,
                projects::Entity::listing(None, false).all(db).await?,
                resources::Entity::listing(None, None, false)
                    .all(db)
                    .await?,
            )
        };
        let tasks = tasks::Entity::listing(None, (!scope.done).then_some(false))
            .all(db)
            .await?;
        Ok(Self {
            areas,
            projects,
            tasks,
            resources,
        })
    }

    /// Projects whose area is in the snapshot (an active project of an
    /// archived area has nowhere to hang when the archive is left out).
    fn placed_projects(&self) -> impl Iterator<Item = &projects::Model> {
        self.projects
            .iter()
            .filter(|p| self.areas.iter().any(|a| a.id == p.area_id))
    }

    fn has_project(&self, id: i64) -> bool {
        self.placed_projects().any(|p| p.id == id)
    }

    fn has_area(&self, id: i64) -> bool {
        self.areas.iter().any(|a| a.id == id)
    }

    /// Where a resource lives: its project, else its area, else nowhere (unfiled).
    fn resource_home(&self, r: &resources::Model) -> Option<Home> {
        r.project_id
            .filter(|id| self.has_project(*id))
            .map(Home::Project)
            .or_else(|| r.area_id.filter(|id| self.has_area(*id)).map(Home::Area))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Home {
    Area(i64),
    Project(i64),
}

impl Home {
    fn id(self) -> String {
        match self {
            Self::Area(id) => area_id(id),
            Self::Project(id) => project_id(id),
        }
    }
}

// Stable ids shared by both formats (archify ids must match `^[a-zA-Z][a-zA-Z0-9_-]*$`).
fn area_id(id: i64) -> String {
    format!("area-{id}")
}
fn project_id(id: i64) -> String {
    format!("project-{id}")
}
fn task_id(id: i64) -> String {
    format!("task-{id}")
}
fn resource_id(id: i64) -> String {
    format!("resource-{id}")
}
