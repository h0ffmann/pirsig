//! arXiv, RSS and GitHub ingest, without the network: papers, feed items and
//! repositories go straight to `store`, as `fetch` would hand them over.
use loco_rs::{testing::prelude::*, TestServer};
use pirsig_github::{
    index::{plan, RepoFacts},
    scope::{Private, Scope},
};
use sea_orm::prelude::Date;
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::{
    app::App,
    ingest::{
        arxiv::{self, Filing, Paper},
        github::{self, Repo},
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

fn repo(name: &str, description: Option<&str>, upstream: Option<&str>, private: bool) -> RepoFacts {
    RepoFacts {
        owner: "h0ffmann".into(),
        name: name.into(),
        url: format!("https://github.com/h0ffmann/{name}"),
        private,
        fork: upstream.is_some(),
        parent: upstream.map(Into::into),
        description: description.map(Into::into),
        topics: vec![],
        language: None,
        languages: vec![],
        stars: 0,
        prs: 0,
        merged_prs: 0,
        pushed_at: Some("2026-03-04T00:00:00Z".into()),
    }
}

#[tokio::test]
#[serial]
async fn github_repos_become_notes_indexed_by_category() {
    request::<App, _, _>(|request, ctx| async move {
        let ocean = area(&request, "Oceanography").await;
        // a repo added by hand before the import: claimed, not duplicated
        request
            .post("/api/resources")
            .json(&json!({ "title": "WW3 by hand", "url": "https://github.com/h0ffmann/WW3" }))
            .await;

        let facts = vec![
            repo("WW3", Some("WAVEWATCH III, a wave model"), Some("NOAA-EMC/WW3"), false),
            repo("ww3-gpu", Some("secret plans"), None, true),
            repo("rust-gpu", Some("Rust for GPU shaders"), Some("Rust-GPU/rust-gpu"), false),
        ];
        let scope = Scope {
            own_private: Private::Minimal,
            ..Scope::for_user("h0ffmann")
        };
        let repos: Vec<Repo> = plan(&scope, &facts).into_iter().map(Repo::from).collect();
        assert_eq!(plan(&Scope::for_user("h0ffmann"), &facts).len(), 2, "private: skipped by default");
        let done = github::store(&ctx.db, repos.clone(), &github::Filing::default())
            .await
            .unwrap();
        assert_eq!(done.len(), 3);
        assert_eq!(
            done.iter().map(|d| d.repo.as_str()).collect::<Vec<_>>(),
            ["h0ffmann/WW3", "h0ffmann/rust-gpu", "h0ffmann/ww3-gpu"],
            "plan sorts by key"
        );
        assert!(!done[0].created, "matched by URL");
        assert_eq!(done[0].resource_id, 1);
        assert_eq!(
            done.iter().map(|d| d.category).collect::<Vec<_>>(),
            ["ocean-waves", "gpu", "ocean-waves"]
        );
        assert_eq!(done[2].area.as_deref(), Some("Oceanography"));
        assert_eq!(done[1].area, None, "no Software area yet: left unfiled");
        assert!(done.iter().all(|d| d.note.is_some()), "every repo gets a note");

        let r: Value = request.get("/api/resources/1").await.json();
        assert_eq!(r["external_id"], "gh:h0ffmann/WW3");
        let r: Value = request
            .get(&format!("/api/resources/{}", done[2].resource_id))
            .await
            .json();
        assert_eq!(r["kind"], "repo");
        assert_eq!(r["area_id"], ocean);

        let zid = done[2].note.as_deref().unwrap();
        let note: Value = request.get(&format!("/api/notes/{zid}")).await.json();
        assert_eq!(note["kind"], "literature");
        assert_eq!(note["title"], "Repo: ww3-gpu");
        assert_eq!(note["resource_id"], done[2].resource_id);
        assert!(!note["body"].as_str().unwrap().contains("secret"), "minimal depth");
        assert_eq!(
            note["tags"],
            json!(["github", "own", "private", "repo/ocean-waves"])
        );
        let hub_zid = note["backlinks"][0]["zid"].as_str().unwrap().to_string();
        let hub: Value = request.get(&format!("/api/notes/{hub_zid}")).await.json();
        assert_eq!(hub["kind"], "hub");
        assert_eq!(hub["title"], "Repos: ocean and wave modelling");
        let (ww3, gpu) = (done[0].note.as_deref().unwrap(), zid);
        assert_eq!(
            hub["body"],
            format!(
                "Every GitHub repository in this category.\n\n## Own repositories\n\n- [[{gpu}]] ww3-gpu (private)\n\n## Forks\n\n- [[{ww3}]] WW3\n"
            )
        );
        let root: Value = request
            .get(&format!("/api/notes/{}", hub["backlinks"][0]["zid"].as_str().unwrap()))
            .await
            .json();
        assert_eq!(root["title"], github::ROOT_HUB);
        assert_eq!(root["links"].as_array().unwrap().len(), 2, "one per category");

        // again, with an area for the rest: nothing new, the unfiled one filed
        let software = area(&request, "Software").await;
        let again = github::store(&ctx.db, repos, &github::Filing::default())
            .await
            .unwrap();
        assert!(again.iter().all(|d| !d.created && d.note.is_none()));
        assert_eq!(again[1].area.as_deref(), Some("Software"));
        let all: Value = request.get("/api/resources").await.json();
        assert_eq!(all["total_items"], 3);
        let r: Value = request
            .get(&format!("/api/resources/{}", done[1].resource_id))
            .await
            .json();
        assert_eq!(r["area_id"], software);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn a_rerun_brings_old_repo_notes_up_to_date() {
    request::<App, _, _>(|request, ctx| async move {
        // notes from an earlier import: a private repo described in full, and
        // a public one without topics; both with a category chosen by hand
        let old = |name: &str, body: &str, tags: Value| {
            let request = &request;
            let (name, body) = (name.to_string(), body.to_string());
            async move {
                let r: Value = request
                    .post("/api/resources")
                    .json(&json!({ "title": format!("h0ffmann/{name}"), "kind": "repo",
                        "url": format!("https://github.com/h0ffmann/{name}"), "body": "our secret" }))
                    .await
                    .json();
                let n = request
                    .post("/api/notes")
                    .json(&json!({ "title": format!("Repo: {name}"), "kind": "literature",
                        "resource_id": r["id"], "body": body, "tags": tags }))
                    .await;
                assert_eq!(n.status_code(), 201, "{}", n.text());
                (r["id"].as_i64().unwrap(), n.json::<Value>()["zid"].as_str().unwrap().to_string())
            }
        };
        let (vault, vault_zid) = old(
            "vault",
            "**h0ffmann/vault**: our secret\n\nREADME opens with: “the secret plan”\n",
            json!(["github", "own", "private", "repo/finance", "idea"]),
        )
        .await;
        let (_, notes_zid) = old(
            "smart-notes",
            "**h0ffmann/smart-notes**: notes\n\nREADME opens with: “A slip-box in Rust.”\n",
            json!(["github", "own", "repo/knowledge"]),
        )
        .await;

        let mut vault_facts = repo("vault", Some("our secret"), None, true);
        vault_facts.topics = vec!["secret-topic".into()];
        let mut smart = repo("smart-notes", Some("notes"), None, false);
        smart.topics = vec!["zettelkasten".into(), "rust".into(), "notes".into()];
        smart.language = Some("Rust".into());
        smart.languages = vec!["Rust".into(), "TypeScript".into()];
        let mut zk = repo("zettelkasten", Some("a second brain"), None, false);
        zk.topics = vec!["rust".into(), "zettelkasten".into()];
        let scope = Scope {
            own_private: Private::Minimal,
            ..Scope::for_user("h0ffmann")
        };
        let repos: Vec<Repo> = plan(&scope, &[vault_facts, smart, zk])
            .into_iter()
            .map(Repo::from)
            .collect();
        let done = github::store(&ctx.db, repos.clone(), &github::Filing::default())
            .await
            .unwrap();
        assert_eq!(
            done.iter().map(|d| (d.repo.as_str(), d.created, d.updated)).collect::<Vec<_>>(),
            [
                ("h0ffmann/smart-notes", false, true),
                ("h0ffmann/vault", false, true),
                ("h0ffmann/zettelkasten", true, false),
            ]
        );

        // the private one, down to minimal depth: no words, no topics
        let note: Value = request.get(&format!("/api/notes/{vault_zid}")).await.json();
        let body = note["body"].as_str().unwrap();
        assert!(!body.contains("secret"), "{body}");
        assert_eq!(note["tags"], json!(["github", "idea", "own", "private", "repo/finance"]));
        let r: Value = request.get(&format!("/api/resources/{vault}")).await.json();
        assert_eq!(r["body"], Value::Null, "the resource keeps no words either");

        // the public one: topics, languages, its README kept, linked to the
        // repo it shares two topics with, still in the category chosen by hand
        let zk_zid = done[2].note.as_deref().unwrap();
        let note: Value = request.get(&format!("/api/notes/{notes_zid}")).await.json();
        assert_eq!(
            note["body"],
            format!(
                "**h0ffmann/smart-notes**: notes\n\nlanguages Rust, TypeScript; public; own repository; \
                 last push 2026-03-04.\n\nTopics: zettelkasten, rust, notes.\n\n\
                 README opens with: “A slip-box in Rust.”\n\n\
                 Category: notes, writing and knowledge tools.\n\nRelated: [[{zk_zid}]] zettelkasten.\n"
            )
        );
        assert_eq!(
            note["tags"],
            json!(["github", "own", "repo/knowledge", "topic/notes", "topic/rust", "topic/zettelkasten"])
        );
        assert_eq!(note["links"][0]["zid"], zk_zid);
        let zk: Value = request.get(&format!("/api/notes/{zk_zid}")).await.json();
        assert_eq!(zk["links"].as_array().unwrap().len(), 1, "and back");

        // once up to date, a rerun changes nothing
        let again = github::store(&ctx.db, repos, &github::Filing::default())
            .await
            .unwrap();
        assert!(again.iter().all(|d| !d.created && !d.updated && d.note.is_none()));
    })
    .await;
}
