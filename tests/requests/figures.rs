use loco_rs::{testing::prelude::*, TestServer};
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::app::App;

async fn post(request: &TestServer, path: &str, body: Value) -> i64 {
    request.post(path).json(&body).await.json::<Value>()["id"]
        .as_i64()
        .unwrap()
}

fn parent_of<'a>(tree: &'a Value, id: &str) -> Option<&'a str> {
    tree["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .map(|n| n["parent"].as_str().unwrap_or("-"))
}

#[tokio::test]
#[serial]
async fn exports_para_as_archify_tree_and_graphify_graph() {
    request::<App, _, _>(|request, _ctx| async move {
        let area = post(&request, "/api/areas", json!({ "name": "Health" })).await;
        let project = post(
            &request,
            "/api/projects",
            json!({ "name": "Run a marathon", "area_id": area, "status": "active" }),
        )
        .await;
        let task = post(
            &request,
            "/api/tasks",
            json!({ "title": "Buy shoes", "project_id": project, "due_on": "2026-11-01" }),
        )
        .await;
        let done = post(
            &request,
            "/api/tasks",
            json!({ "title": "Sign up", "project_id": project }),
        )
        .await;
        request
            .patch(&format!("/api/tasks/{done}"))
            .json(&json!({ "done": true }))
            .await;
        let in_project = post(
            &request,
            "/api/resources",
            json!({ "title": "Couch to 5k", "project_id": project, "area_id": area }),
        )
        .await;
        let unfiled = post(&request, "/api/resources", json!({ "title": "Inbox" })).await;
        let old = post(&request, "/api/areas", json!({ "name": "Old hobby" })).await;
        request.post(&format!("/api/areas/{old}/archive")).await;

        let tree: Value = request.get("/api/figures/tree").await.json();
        assert_eq!(tree["schema_version"], 1);
        assert_eq!(tree["diagram_type"], "tree");
        assert_eq!(tree["meta"]["output"], "para.html");
        assert_eq!(parent_of(&tree, "brain"), Some("-"));
        assert_eq!(parent_of(&tree, &format!("area-{area}")), Some("brain"));
        assert_eq!(
            parent_of(&tree, &format!("project-{project}")),
            Some(format!("area-{area}").as_str())
        );
        assert_eq!(
            parent_of(&tree, &format!("task-{task}")),
            Some(format!("project-{project}").as_str())
        );
        assert_eq!(
            parent_of(&tree, &format!("resource-{in_project}")),
            Some(format!("project-{project}").as_str())
        );
        assert_eq!(
            parent_of(&tree, &format!("resource-{unfiled}")),
            Some("unfiled")
        );
        assert_eq!(parent_of(&tree, "unfiled"), Some("brain"));
        // done tasks and the archive stay out unless asked for
        assert_eq!(parent_of(&tree, &format!("task-{done}")), None);
        assert_eq!(parent_of(&tree, &format!("area-{old}")), None);

        let tree: Value = request
            .get("/api/figures/tree?archived=true&done=true")
            .await
            .json();
        assert_eq!(
            parent_of(&tree, &format!("task-{done}")),
            Some(format!("project-{project}").as_str())
        );
        let old_node = tree["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == format!("area-{old}"))
            .unwrap();
        assert_eq!(old_node["sublabel"], "archived");

        let graph: Value = request.get("/api/figures/graph").await.json();
        assert_eq!(graph["directed"], true);
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 5);
        let links = graph["links"].as_array().unwrap();
        assert_eq!(links.len(), 3);
        assert!(links.iter().all(|l| l["relation"] == "contains"
            && l["confidence"] == "EXTRACTED"
            && l["source_file"].as_str().unwrap().starts_with("pirsig/")));
        assert!(links
            .iter()
            .any(|l| l["source"] == format!("project-{project}")
                && l["target"] == format!("resource-{in_project}")));
        let unfiled_node = graph["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == format!("resource-{unfiled}"))
            .unwrap();
        assert_eq!(unfiled_node["file_type"], "document");
        assert_eq!(
            unfiled_node["source_file"],
            format!("pirsig/resources/{unfiled}")
        );
    })
    .await;
}
