//! The brain goes out as an Obsidian vault and comes back the same.
use loco_rs::{app::Hooks, testing::prelude::*, TestServer};
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::{app::App, vault};

async fn post(request: &TestServer, path: &str, body: Value) -> Value {
    let res = request.post(path).json(&body).await;
    assert!(res.status_code().is_success(), "{path}: {}", res.text());
    res.json()
}

fn read(dir: &std::path::Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[tokio::test]
#[serial]
async fn round_trips_para_and_the_slipbox() {
    request::<App, _, _>(|request, ctx| async move {
        let weather = post(&request, "/api/areas", json!({ "name": "Weather" })).await["id"].clone();
        let bench = post(
            &request,
            "/api/projects",
            json!({ "name": "Buoy bench", "area_id": weather, "due_on": "2027-01-31" }),
        )
        .await["id"]
            .clone();
        post(
            &request,
            "/api/tasks",
            json!({ "title": "Fetch buoys", "project_id": bench, "due_on": "2026-11-15" }),
        )
        .await;
        let paper = post(
            &request,
            "/api/resources",
            json!({ "title": "WeatherBench", "kind": "paper", "citekey": "rasp2020weatherbench", "project_id": bench, "area_id": weather }),
        )
        .await["id"]
            .clone();
        let old = post(&request, "/api/areas", json!({ "name": "Old hobby" })).await["id"].clone();
        request.post(&format!("/api/areas/{old}/archive")).await;

        post(
            &request,
            "/api/notes",
            json!({ "title": "Index", "kind": "hub", "body": "Start at [[2]], see [[Atomic idea#Why|why]]." }),
        )
        .await;
        post(&request, "/api/notes", json!({ "title": "Atomic idea", "kind": "permanent", "body": "One idea. #method" })).await;
        post(
            &request,
            "/api/notes",
            json!({ "title": "Benchmarks need a fixed protocol", "kind": "literature", "resource_id": paper }),
        )
        .await;
        post(&request, "/api/notes/2/links", json!({ "target": "1", "context": "filed under" })).await;
        post(&request, &format!("/api/projects/{bench}/notes"), json!({ "zid": "2", "context": "the method" })).await;

        let dir = std::env::temp_dir().join(format!("ztk-vault-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Zettelkasten")).unwrap();
        std::fs::write(dir.join("Mine.md"), "# My own\n\nwritten in Obsidian, links [[2 Atomic idea]]\n").unwrap();
        std::fs::write(dir.join("Zettelkasten/9 Gone.md"), "---\nztk: note\nzid: \"9\"\n---\n").unwrap();

        let out = vault::export(&ctx.db, &dir, true).await.unwrap();
        assert_eq!((out.notes, out.areas, out.projects, out.resources, out.removed), (3, 2, 1, 1, 1));
        assert!(dir.join("Mine.md").exists(), "never touches files ztk did not write");
        let index = read(&dir, "Zettelkasten/1 Index.md");
        assert!(index.contains("Start at [[2 Atomic idea|2]], see [[2 Atomic idea#Why|why]]."), "{index}");
        let atomic = read(&dir, "Zettelkasten/2 Atomic idea.md");
        assert!(atomic.contains("- [[1 Index|1]] Index: filed under"), "{atomic}");
        let project = read(&dir, "Projects/Buoy bench.md");
        assert!(project.contains("area: \"[[Areas/Weather]]\""), "{project}");
        assert!(project.contains("- [ ] Fetch buoys 📅 2026-11-15"), "{project}");
        assert!(project.contains("- [[2 Atomic idea|2]]: the method"), "{project}");
        assert!(read(&dir, "Archive/Areas/Old hobby.md").contains("archived: true"));
        assert!(read(&dir, "Zettelkasten/3 Benchmarks need a fixed protocol.md")
            .contains("resource: \"[[Resources/WeatherBench]]\""));

        // into an empty brain
        App::truncate(&ctx).await.unwrap();
        let dry = vault::import(&ctx.db, &dir, true).await.unwrap();
        assert!(dry.dry_run);
        assert_eq!(dry.notes_created, 4);
        let page: Value = request.get("/api/notes").await.json();
        assert_eq!(page["total_items"], 0, "a dry run writes nothing");

        let report = vault::import(&ctx.db, &dir, false).await.unwrap();
        assert_eq!((report.notes_created, report.areas, report.projects, report.tasks, report.resources), (4, 2, 1, 1, 1));
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);

        let index: Value = request.get("/api/notes/1").await.json();
        assert_eq!(index["body"], "Start at [[2]], see [[2#Why|why]].");
        assert_eq!(index["kind"], "hub");
        let atomic: Value = request.get("/api/notes/2").await.json();
        assert_eq!(atomic["tags"], json!(["method"]));
        assert_eq!(atomic["links"][0]["zid"], "1");
        assert_eq!(atomic["links"][0]["context"], "filed under");
        let lit: Value = request.get("/api/notes/3").await.json();
        assert_eq!(lit["source"], "@rasp2020weatherbench");
        assert!(!lit["resource_id"].is_null());
        let mine: Value = request.get("/api/notes/4").await.json();
        assert_eq!(mine["title"], "My own");
        assert_eq!(mine["links"][0]["zid"], "2", "an Obsidian file-stem link resolves");

        let projects: Value = request.get("/api/projects").await.json();
        let project = &projects["items"][0];
        assert_eq!(project["name"], "Buoy bench");
        assert_eq!(project["due_on"], "2027-01-31");
        let pulled: Value = request.get(&format!("/api/projects/{}/notes", project["id"])).await.json();
        assert_eq!(pulled[0]["context"], "the method");
        let tasks: Value = request.get("/api/tasks").await.json();
        assert_eq!(tasks["items"][0]["due_on"], "2026-11-15");
        let archived: Value = request.get("/api/areas?archived=true").await.json();
        assert_eq!(archived["items"][0]["name"], "Old hobby");

        // again: everything is matched, nothing doubles
        let again = vault::import(&ctx.db, &dir, false).await.unwrap();
        assert_eq!((again.notes_created, again.notes_updated), (0, 4));
        let page: Value = request.get("/api/notes").await.json();
        assert_eq!(page["total_items"], 4);
        let tasks: Value = request.get("/api/tasks").await.json();
        assert_eq!(tasks["total_items"], 1);

        let _ = std::fs::remove_dir_all(&dir);
    })
    .await;
}
