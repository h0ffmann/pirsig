//! arXiv and RSS ingest, without the network: papers and feed items go
//! straight to `store`, as `fetch` would hand them over.
use loco_rs::{testing::prelude::*, TestServer};
use sea_orm::prelude::Date;
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::{
    app::App,
    ingest::{
        arxiv::{self, Filing, Paper},
        rss,
    },
    models::feeds,
};

async fn area(request: &TestServer, name: &str) -> i64 {
    let res = request
        .post("/api/areas")
        .json(&json!({ "name": name }))
        .await;
    res.json::<Value>()["id"].as_i64().unwrap()
}

fn paper(id: &str, title: &str, author: &str, categories: &[&str]) -> Paper {
    Paper {
        id: id.into(),
        title: title.into(),
        authors: vec![author.into()],
        summary: "An abstract.".into(),
        published: Some(Date::from_ymd_opt(2024, 3, 1).unwrap()),
        categories: categories.iter().map(|c| (*c).to_string()).collect(),
    }
}

#[tokio::test]
#[serial]
async fn arxiv_papers_become_cited_filed_resources() {
    request::<App, _, _>(|request, ctx| async move {
        let game = area(&request, "Game Theory").await;
        area(&request, "Weather").await;
        area(&request, "Oceanography").await;

        let nash = paper(
            "2403.00001",
            "On Equilibria in Auctions",
            "John Nash",
            &["cs.GT"],
        );
        let waves = paper(
            "2403.00002",
            "The Rogue Wave Climate",
            "Ana Sá",
            &["physics.ao-ph"],
        );
        let done = arxiv::store(
            &ctx.db,
            vec![nash.clone(), waves],
            &Filing {
                note: true,
                ..Filing::default()
            },
        )
        .await
        .unwrap();
        assert!(done[0].created);
        assert_eq!(done[0].citekey.as_deref(), Some("nash2024equilibria"));
        assert_eq!(
            done[0].area.as_deref(),
            Some("Game Theory"),
            "cs.GT has one home"
        );
        assert_eq!(done[1].citekey.as_deref(), Some("sa2024rogue"));
        assert_eq!(
            done[1].area, None,
            "ao-ph fits two areas: left for the person"
        );
        assert_eq!(done[1].could_go_in, ["Weather", "Oceanography"]);

        let r: Value = request
            .get(&format!("/api/resources/{}", done[0].resource_id))
            .await
            .json();
        assert_eq!(r["kind"], "paper");
        assert_eq!(r["url"], "https://arxiv.org/abs/2403.00001");
        assert_eq!(r["external_id"], "arxiv:2403.00001");
        assert_eq!(r["area_id"], game);

        // the inbox note points at the paper and cites it
        let zid = done[0].note.clone().unwrap();
        let note: Value = request.get(&format!("/api/notes/{zid}")).await.json();
        assert_eq!(note["kind"], "fleeting");
        assert_eq!(note["source"], "@nash2024equilibria");

        // a new version updates in place, keeps the citekey, makes no second inbox note
        let v2 = Paper {
            title: "On Equilibria in Auctions (revised)".into(),
            ..nash
        };
        let again = arxiv::store(
            &ctx.db,
            vec![v2],
            &Filing {
                note: true,
                ..Filing::default()
            },
        )
        .await
        .unwrap();
        assert!(!again[0].created);
        assert_eq!(again[0].resource_id, done[0].resource_id);
        assert_eq!(again[0].note, None);
        let all: Value = request.get("/api/resources").await.json();
        assert_eq!(all["total_items"], 2);

        // another Nash paper of 2024 on equilibria gets the next free key
        let other = paper("2403.00003", "Equilibria, again", "J. Nash", &[]);
        let third = arxiv::store(&ctx.db, vec![other], &Filing::default())
            .await
            .unwrap();
        assert_eq!(third[0].citekey.as_deref(), Some("nash2024equilibriab"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn arxiv_respects_an_explicit_project() {
    request::<App, _, _>(|request, ctx| async move {
        let weather = area(&request, "Weather").await;
        let project = request
            .post("/api/projects")
            .json(&json!({ "name": "Buoy bench", "area_id": weather }))
            .await
            .json::<Value>()["id"]
            .as_i64()
            .unwrap();
        let p = paper(
            "2403.00004",
            "Scoring forecasts",
            "Tilmann Gneiting",
            &["cs.GT"],
        );
        let filing = Filing {
            project_id: Some(project),
            ..Filing::default()
        };
        let done = arxiv::store(&ctx.db, vec![p], &filing).await.unwrap();
        assert_eq!(
            done[0].area.as_deref(),
            Some("Weather"),
            "the project decides"
        );
        let r: Value = request
            .get(&format!("/api/resources/{}", done[0].resource_id))
            .await
            .json();
        assert_eq!(r["project_id"], project);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn rss_entries_become_resources_and_notes_go_out() {
    request::<App, _, _>(|request, ctx| async move {
        let ocean = area(&request, "Oceanography").await;
        let res = request
            .post("/api/feeds")
            .json(&json!({ "url": "https://example.org/feed.xml", "area_id": ocean }))
            .await;
        assert_eq!(res.status_code(), 201);
        let res = request.post("/api/feeds").json(&json!({ "url": "nope" })).await;
        assert_eq!(res.status_code(), 400);

        let feed = feeds::Model::find(&ctx.db, 1).await.unwrap();
        let xml = br#"<rss version="2.0"><channel><title>Waves</title>
<item><title>Rogue waves</title><link>https://example.org/rogue</link><guid>g1</guid><description>Big</description></item>
</channel></rss>"#;
        let (_, items) = rss::parse(xml).unwrap();
        assert_eq!(rss::store(&ctx.db, &feed, items.clone()).await.unwrap(), (1, 0));
        assert_eq!(rss::store(&ctx.db, &feed, items).await.unwrap(), (0, 1), "same guid, same resource");
        let all: Value = request.get("/api/resources").await.json();
        assert_eq!(all["total_items"], 1);
        assert_eq!(all["items"][0]["area_id"], ocean);
        assert_eq!(all["items"][0]["kind"], "article");

        // out: permanent notes by default, never the inbox
        request.post("/api/notes").json(&json!({ "title": "Waves <break> & bend", "kind": "permanent", "body": "#ocean" })).await;
        request.post("/api/notes").json(&json!({ "title": "half a thought" })).await;
        let res = request.get("/api/slipbox/feed.xml").await;
        assert_eq!(res.status_code(), 200);
        let xml = res.text();
        assert!(xml.contains("<title>Waves &lt;break&gt; &amp; bend</title>"), "{xml}");
        assert!(xml.contains("<category>ocean</category>"));
        assert!(xml.contains("<guid isPermaLink=\"false\">ztk:1</guid>"));
        assert!(!xml.contains("half a thought"));
        assert!(feed_rs::parser::parse(xml.as_bytes()).is_ok(), "valid RSS");
        let res = request.get("/api/slipbox/feed.xml?kind=fleeting").await;
        assert_eq!(res.status_code(), 400);
    })
    .await;
}
