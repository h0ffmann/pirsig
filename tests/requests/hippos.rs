//! Hippo identities: their notes form a rhizome that stays whole.
use loco_rs::{testing::prelude::*, TestServer};
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::app::App;

async fn note(request: &TestServer, body: Value) -> Value {
    let res = request.post("/api/notes").json(&body).await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    res.json()
}

fn reasons(links: &Value) -> Vec<(String, String)> {
    links
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["zid"].as_str().unwrap().to_string(),
                l["context"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect()
}

async fn pieces(request: &TestServer, hippo: i64) -> usize {
    let map: Value = request
        .get(&format!("/api/hippos/{hippo}/map"))
        .await
        .json();
    map["components"].as_array().unwrap().len()
}

#[tokio::test]
#[serial]
async fn a_hippo_grows_a_rhizome_that_heals() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/hippos")
            .json(&json!({ "name": "h0", "traits": ["bipolar-ish"] }))
            .await;
        assert_eq!(res.status_code(), 400, "traits are a known, self-described set");
        let res = request
            .post("/api/hippos")
            .json(&json!({ "name": "h0", "traits": ["ADHD", "autistic"], "fixations": ["Rogue Waves", "#chess"] }))
            .await;
        assert_eq!(res.status_code(), 201);
        let hippo: Value = res.json();
        assert_eq!(hippo["traits"], json!(["adhd", "autistic"]));
        assert_eq!(hippo["fixations"], json!(["rogue waves", "chess"]));
        let h = hippo["id"].as_i64().unwrap();

        let a = note(&request, json!({ "title": "Rogue waves at sea", "hippo_id": h })).await;
        assert_eq!(a["links"], json!([]), "the first note has nothing to join yet");
        // nothing in common: it joins the stream where it was last
        let b = note(&request, json!({ "title": "Sicilian openings", "body": "#chess", "hippo_id": h })).await;
        assert_eq!(reasons(&b["links"]), [("1".into(), "rhizome: captured next to \"Rogue waves at sea\"".into())]);
        // a shared fixation is the strongest tie
        let c = note(&request, json!({ "title": "More rogue waves, in data", "hippo_id": h })).await;
        assert_eq!(reasons(&c["links"]), [("1".into(), "rhizome: both circle rogue waves".into())]);

        let res = request
            .post("/api/notes")
            .json(&json!({ "title": "Under a", "after": "1", "hippo_id": h }))
            .await;
        assert_eq!(res.status_code(), 400, "a rhizome has no trunk");

        // outside the Hippo, notes are left alone
        let loose = note(&request, json!({ "title": "Grocery list" })).await;
        assert_eq!(loose["links"], json!([]));

        let map: Value = request.get(&format!("/api/hippos/{h}/map")).await.json();
        assert_eq!(map["notes"], 3);
        assert_eq!(map["components"].as_array().unwrap().len(), 1);
        assert_eq!(map["plateaus"][0]["fixation"], "rogue waves");
        assert_eq!(map["plateaus"][0]["notes"].as_array().unwrap().len(), 2);
        assert_eq!(map["plateaus"][1]["notes"][0]["zid"], "2", "#chess joins the chess plateau");
        assert_eq!(map["resume"][0]["zid"], "3", "where you left off comes first");

        // deleting the note in the middle reconnects the ones around it
        assert_eq!(request.delete("/api/notes/1").await.status_code(), 204);
        let b: Value = request.get("/api/notes/2").await.json();
        assert_eq!(reasons(&b["links"]), [("3".into(), "rhizome: reconnected through \"Rogue waves at sea\"".into())]);
        assert_eq!(pieces(&request, h).await, 1);

        // a cut the person makes splits it; weave makes it whole again
        request.delete("/api/notes/2/links/3").await;
        assert_eq!(pieces(&request, h).await, 2);
        let woven: Value = request.post(&format!("/api/hippos/{h}/weave")).await.json();
        assert!(woven["woven"].as_u64().unwrap() + woven["bridged"].as_u64().unwrap() >= 1);
        assert_eq!(pieces(&request, h).await, 1);

        // joining a Hippo later weaves the note in
        let joined: Value = request
            .patch(&format!("/api/notes/{}", loose["zid"].as_str().unwrap()))
            .json(&json!({ "hippo_id": h }))
            .await
            .json();
        assert_eq!(joined["hippo_id"], h);
        assert!(!joined["links"].as_array().unwrap().is_empty());

        let ready: Value = request.get("/api/readiness").await.json();
        let section = &ready["sections"][5];
        assert_eq!(section["name"], "Hippo rhizome");
        assert_eq!(section["checks"][0]["status"], "ready");

        // the identity can go; the notes and their links stay
        assert_eq!(request.delete(&format!("/api/hippos/{h}")).await.status_code(), 204);
        let c: Value = request.get("/api/notes/3").await.json();
        assert!(c["hippo_id"].is_null());
        assert!(!c["links"].as_array().unwrap().is_empty() || !c["backlinks"].as_array().unwrap().is_empty());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn a_rhizome_survives_the_obsidian_round_trip() {
    use loco_rs::app::Hooks;
    request::<App, _, _>(|request, ctx| async move {
        let h = request
            .post("/api/hippos")
            .json(&json!({ "name": "h0", "fixations": ["chess"] }))
            .await
            .json::<Value>()["id"]
            .as_i64()
            .unwrap();
        note(
            &request,
            json!({ "title": "Chess as a language", "hippo_id": h }),
        )
        .await;
        note(
            &request,
            json!({ "title": "Endgames are chess grammar", "hippo_id": h }),
        )
        .await;
        let dir = std::env::temp_dir().join(format!("pirsig-hippo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        zettelkasten::vault::export(&ctx.db, &dir, false)
            .await
            .unwrap();
        let md = std::fs::read_to_string(dir.join("Zettelkasten/2 Endgames are chess grammar.md"))
            .unwrap();
        assert!(md.contains("hippo: \"h0\""), "{md}");
        assert!(md.contains("rhizome: both circle chess"), "{md}");

        App::truncate(&ctx).await.unwrap();
        let report = zettelkasten::vault::import(&ctx.db, &dir, false)
            .await
            .unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("Hippo \"h0\" created")));
        let hippos: Value = request.get("/api/hippos").await.json();
        let h = hippos[0]["id"].as_i64().unwrap();
        let map: Value = request.get(&format!("/api/hippos/{h}/map")).await.json();
        assert_eq!(map["notes"], 2);
        assert_eq!(map["links"], 1, "the link comes back once, not woven twice");
        let _ = std::fs::remove_dir_all(&dir);
    })
    .await;
}
