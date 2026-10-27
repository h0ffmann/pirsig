use loco_rs::testing::prelude::*;
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::app::App;

#[tokio::test]
#[serial]
async fn can_crud_areas_projects_tasks() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/areas")
            .json(&json!({ "name": "Health" }))
            .await;
        assert_eq!(res.status_code(), 201);
        let area: Value = res.json();
        let area_id = area["id"].as_i64().unwrap();

        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "Run a marathon", "area_id": area_id, "status": "active" }))
            .await;
        assert_eq!(res.status_code(), 201);
        let project_id = res.json::<Value>()["id"].as_i64().unwrap();

        let res = request
            .post("/api/tasks")
            .json(
                &json!({ "title": "Buy shoes", "project_id": project_id, "due_on": "2026-11-01" }),
            )
            .await;
        assert_eq!(res.status_code(), 201);
        let task: Value = res.json();
        assert_eq!(task["done"], false);
        let task_id = task["id"].as_i64().unwrap();

        // partial update only touches the given fields
        let res = request
            .patch(&format!("/api/tasks/{task_id}"))
            .json(&json!({ "done": true }))
            .await;
        assert_eq!(res.status_code(), 200);
        let task: Value = res.json();
        assert_eq!(task["done"], true);
        assert_eq!(task["title"], "Buy shoes");

        let res = request
            .get(&format!("/api/tasks?project_id={project_id}&done=false"))
            .await;
        assert_eq!(res.json::<Value>()["total_items"], 0);

        let res = request
            .put(&format!("/api/areas/{area_id}"))
            .json(&json!({ "name": "Fitness" }))
            .await;
        assert_eq!(res.json::<Value>()["name"], "Fitness");

        let res = request.delete(&format!("/api/tasks/{task_id}")).await;
        assert_eq!(res.status_code(), 204);
        let res = request.get(&format!("/api/tasks/{task_id}")).await;
        assert_eq!(res.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn rejects_children_of_missing_parents() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "Orphan", "area_id": 999 }))
            .await;
        assert_eq!(res.status_code(), 404);

        let res = request
            .post("/api/tasks")
            .json(&json!({ "title": "Orphan", "project_id": 999 }))
            .await;
        assert_eq!(res.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn archive_hides_from_default_listing() {
    request::<App, _, _>(|request, _ctx| async move {
        let area_id = request
            .post("/api/areas")
            .json(&json!({ "name": "Old hobby" }))
            .await
            .json::<Value>()["id"]
            .as_i64()
            .unwrap();
        let resource_id = request
            .post("/api/resources")
            .json(&json!({ "title": "Notes", "area_id": area_id }))
            .await
            .json::<Value>()["id"]
            .as_i64()
            .unwrap();

        let res = request.post(&format!("/api/areas/{area_id}/archive")).await;
        assert!(!res.json::<Value>()["archived_at"].is_null());
        request
            .post(&format!("/api/resources/{resource_id}/archive"))
            .await;

        let active: Value = request.get("/api/areas").await.json();
        assert_eq!(active["total_items"], 0);
        let archived: Value = request.get("/api/areas?archived=true").await.json();
        assert_eq!(archived["total_items"], 1);
        let resources: Value = request.get("/api/resources?archived=true").await.json();
        assert_eq!(resources["items"][0]["title"], "Notes");

        let res = request
            .post(&format!("/api/areas/{area_id}/unarchive"))
            .await;
        assert!(res.json::<Value>()["archived_at"].is_null());
        let active: Value = request.get("/api/areas").await.json();
        assert_eq!(active["total_items"], 1);
    })
    .await;
}
