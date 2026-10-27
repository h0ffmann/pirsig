use loco_rs::{testing::prelude::*, TestServer};
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::app::App;

async fn create(request: &TestServer, body: Value) -> Value {
    let res = request.post("/api/notes").json(&body).await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    res.json()
}

fn zids(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|n| n["zid"].as_str().unwrap())
        .collect()
}

#[tokio::test]
#[serial]
async fn files_notes_in_folgezettel_order() {
    request::<App, _, _>(|request, _ctx| async move {
        let root = create(&request, json!({ "title": "Zettelkasten", "kind": "hub" })).await;
        assert_eq!(root["zid"], "1");
        assert_eq!(root["kind"], "hub");
        let a = create(&request, json!({ "title": "Atomic", "after": "1" })).await;
        assert_eq!(a["zid"], "1a");
        assert_eq!(a["kind"], "fleeting");
        let b = create(&request, json!({ "title": "Links", "after": "1" })).await;
        assert_eq!(b["zid"], "1b");
        let a1 = create(&request, json!({ "title": "One idea", "after": "1a" })).await;
        assert_eq!(a1["zid"], "1a1");
        let two = create(&request, json!({ "title": "Second train" })).await;
        assert_eq!(two["zid"], "2");

        let tree: Value = request.get("/api/slipbox/tree").await.json();
        assert_eq!(zids(&tree), ["1", "1a", "1a1", "1b", "2"]);
        assert_eq!(tree[2]["depth"], 2);

        let res = request
            .post("/api/notes")
            .json(&json!({ "title": "Lost", "after": "9" }))
            .await;
        assert_eq!(res.status_code(), 404);
        let res = request
            .post("/api/notes")
            .json(&json!({ "title": "Odd", "kind": "bogus" }))
            .await;
        assert_eq!(res.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn wikilinks_and_hashtags_become_links_and_tags() {
    request::<App, _, _>(|request, _ctx| async move {
        create(&request, json!({ "title": "Zettelkasten" })).await;
        // [[Later]] does not exist yet: it is picked up when that note is created
        let note = create(
            &request,
            json!({
                "title": "Atomic notes",
                "body": "See [[Zettelkasten]] and [[Later]]. #PKM #writing",
                "tags": ["method"],
            }),
        )
        .await;
        assert_eq!(note["tags"], json!(["method", "pkm", "writing"]));
        assert_eq!(zids(&note["links"]), ["1"]);
        assert_eq!(note["links"][0]["auto"], true);

        let later = create(&request, json!({ "title": "Later" })).await;
        assert_eq!(zids(&later["backlinks"]), ["2"]);

        let root: Value = request.get("/api/notes/1").await.json();
        assert_eq!(zids(&root["backlinks"]), ["2"]);

        // editing the body re-syncs auto links and hashtags, keeps explicit tags
        let res = request
            .patch("/api/notes/2")
            .json(&json!({ "body": "Only [[3]] now. #writing" }))
            .await;
        let note: Value = res.json();
        assert_eq!(zids(&note["links"]), ["3"]);
        assert_eq!(note["tags"], json!(["method", "pkm", "writing"]));

        let res = request
            .patch("/api/notes/2")
            .json(&json!({ "tags": [] }))
            .await;
        assert_eq!(res.json::<Value>()["tags"], json!(["writing"]));

        let page: Value = request.get("/api/notes?tag=writing").await.json();
        assert_eq!(page["total_items"], 1);
        let page: Value = request.get("/api/notes?q=ONLY").await.json();
        assert_eq!(page["items"][0]["zid"], "2");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn manual_links_carry_a_reason_and_survive_edits() {
    request::<App, _, _>(|request, _ctx| async move {
        create(&request, json!({ "title": "Claim" })).await;
        create(
            &request,
            json!({ "title": "Counterpoint", "body": "mentions [[1]]" }),
        )
        .await;

        let res = request
            .post("/api/notes/1/links")
            .json(&json!({ "target": "2", "context": "is challenged by" }))
            .await;
        let note: Value = res.json();
        assert_eq!(note["links"][0]["context"], "is challenged by");
        assert_eq!(note["links"][0]["auto"], false);

        request
            .patch("/api/notes/1")
            .json(&json!({ "body": "rewritten" }))
            .await;
        let note: Value = request.get("/api/notes/1").await.json();
        assert_eq!(zids(&note["links"]), ["2"]);

        let res = request
            .post("/api/notes/1/links")
            .json(&json!({ "target": "1" }))
            .await;
        assert_eq!(res.status_code(), 400);

        let res = request.delete("/api/notes/1/links/2").await;
        assert_eq!(res.status_code(), 204);
        let res = request.delete("/api/notes/1/links/2").await;
        assert_eq!(res.status_code(), 404);

        let md = request.get("/api/notes/2/markdown").await.text();
        assert!(md.starts_with("---\nzid: \"2\"\ntitle: \"Counterpoint\""));
        assert!(md.contains("# Counterpoint\n\nmentions [[1]]"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn slipbox_views() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/slipbox/random").await;
        assert_eq!(res.status_code(), 404);

        create(
            &request,
            json!({ "title": "Hub", "kind": "hub", "body": "[[2]]" }),
        )
        .await;
        create(&request, json!({ "title": "Linked", "kind": "permanent" })).await;
        create(&request, json!({ "title": "Loose thought" })).await;

        let inbox: Value = request.get("/api/slipbox/inbox").await.json();
        assert_eq!(zids(&inbox), ["3"]);
        let orphans: Value = request.get("/api/slipbox/orphans").await.json();
        assert_eq!(zids(&orphans), ["3"]);
        let graph: Value = request.get("/api/slipbox/graph").await.json();
        assert_eq!(graph["edges"], json!([{ "source": "1", "target": "2" }]));
        let stats: Value = request.get("/api/slipbox/stats").await.json();
        assert_eq!(stats["notes"], 3);
        assert_eq!(stats["links"], 1);
        assert_eq!(stats["inbox"], 1);
        let random = request.get("/api/slipbox/random").await;
        assert_eq!(random.status_code(), 200);

        // deleting a note takes its links with it
        let res = request.delete("/api/notes/2").await;
        assert_eq!(res.status_code(), 204);
        let stats: Value = request.get("/api/slipbox/stats").await.json();
        assert_eq!(stats["links"], 0);
        assert_eq!(stats["orphans"], 2);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn underscores_and_percents_are_literal() {
    request::<App, _, _>(|request, _ctx| async move {
        create(
            &request,
            json!({ "title": "x", "body": "uses [[snake_case names]]" }),
        )
        .await;
        create(&request, json!({ "title": "other", "body": "snakeXcase" })).await;
        // the target arrives after the link: its title has an underscore
        let target = create(&request, json!({ "title": "snake_case names" })).await;
        assert_eq!(
            zids(&target["backlinks"]),
            ["1"],
            "late link with `_` adopted"
        );

        let page: Value = request.get("/api/notes?q=snake_case").await.json();
        let found: Vec<&str> = page["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["zid"].as_str().unwrap())
            .collect();
        assert_eq!(found, ["3", "1"], "`_` is not a wildcard");
        let page: Value = request.get("/api/notes?q=50%25").await.json();
        assert_eq!(page["total_items"], 0);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn code_is_not_links_or_tags_and_anchors_resolve() {
    request::<App, _, _>(|request, _ctx| async move {
        create(&request, json!({ "title": "Target" })).await;
        let note = create(
            &request,
            json!({
                "title": "Mixed",
                "body": "See [[Target#Why]] #real\n\n```\n#include [[Nope]]\n```\n`#nottag` https://x.org/#frag",
            }),
        )
        .await;
        assert_eq!(zids(&note["links"]), ["1"]);
        assert_eq!(note["tags"], json!(["real"]));
        // a note created later named like the link in code stays unlinked
        let nope = create(&request, json!({ "title": "Nope" })).await;
        assert_eq!(nope["backlinks"], json!([]));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn links_by_title_survive_a_rename() {
    request::<App, _, _>(|request, _ctx| async move {
        create(&request, json!({ "title": "Old name" })).await;
        create(
            &request,
            json!({ "title": "Citer", "body": "see [[Old name]] and [[old name|it]]" }),
        )
        .await;
        request
            .patch("/api/notes/1")
            .json(&json!({ "title": "New name" }))
            .await;
        let citer: Value = request.get("/api/notes/2").await.json();
        assert_eq!(citer["body"], "see [[New name]] and [[New name|it]]");
        // saving the citer again keeps the link: its body still resolves
        request
            .patch("/api/notes/2")
            .json(&json!({ "body": format!("{} again", citer["body"].as_str().unwrap()) }))
            .await;
        let citer: Value = request.get("/api/notes/2").await.json();
        assert_eq!(zids(&citer["links"]), ["1"]);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn literature_notes_say_where_they_come_from() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/notes")
            .json(&json!({ "title": "Attention is all you need", "kind": "literature" }))
            .await;
        assert_eq!(res.status_code(), 400, "literature without a source");
        let res = request
            .post("/api/notes")
            .json(&json!({ "title": "Blank", "kind": "permanent" }))
            .await;
        assert_eq!(res.status_code(), 201);
        let res = request.post("/api/notes").json(&json!({ "title": "  " })).await;
        assert_eq!(res.status_code(), 400, "blank title");

        let resource = request
            .post("/api/resources")
            .json(&json!({ "title": "Attention", "kind": "paper", "citekey": "vaswani2017attention" }))
            .await
            .json::<Value>()["id"]
            .clone();
        let note = create(
            &request,
            json!({ "title": "Self-attention replaces recurrence", "kind": "literature", "resource_id": resource }),
        )
        .await;
        assert_eq!(note["source"], "@vaswani2017attention");
        assert_eq!(note["resource_id"], resource);

        // a structure note from another slip-box is a hub here
        let hub = create(&request, json!({ "title": "Index", "kind": "Structure" })).await;
        assert_eq!(hub["kind"], "hub");

        // clearing the source of a literature note is refused; of a permanent one, fine
        let res = request
            .patch(&format!("/api/notes/{}", note["zid"].as_str().unwrap()))
            .json(&json!({ "source": null, "resource_id": null }))
            .await;
        assert_eq!(res.status_code(), 400);
        let res = request
            .patch("/api/notes/1")
            .json(&json!({ "source": "a talk" }))
            .await;
        assert_eq!(res.json::<Value>()["source"], "a talk");
        let res = request
            .patch("/api/notes/1")
            .json(&json!({ "source": null }))
            .await;
        assert!(res.json::<Value>()["source"].is_null());

        // deleting the resource keeps the note and its source
        request
            .delete(&format!("/api/resources/{resource}"))
            .await;
        let note: Value = request
            .get(&format!("/api/notes/{}", note["zid"].as_str().unwrap()))
            .await
            .json();
        assert!(note["resource_id"].is_null());
        assert_eq!(note["source"], "@vaswani2017attention");
    })
    .await;
}
