//! A practice session over the API: next, answer, and the schedule moving.
use loco_rs::{testing::prelude::*, TestServer};
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::{
    app::App,
    plasticity::{plast, Answer},
};

/// The right answer to an exercise the API handed out.
fn right(ex: &Value) -> String {
    let e = (plast(ex["plast"].as_str().unwrap()).unwrap().generate)(
        ex["seed"].as_u64().unwrap(),
        u8::try_from(ex["level"].as_u64().unwrap()).unwrap(),
    );
    match e.answer {
        Answer::Number(n) => n.to_string(),
        Answer::Choice(i) => ["A", "B", "C", "D"][i].to_string(),
        Answer::Text(forms) => forms[0].clone(),
    }
}

async fn answer(request: &TestServer, ex: &Value, given: &str) -> Value {
    let res = request
        .post("/api/plasticity/answer")
        .json(&json!({ "plast": ex["plast"], "seed": ex["seed"], "level": ex["level"], "answer": given, "millis": 1200 }))
        .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    res.json()
}

#[tokio::test]
#[serial]
async fn a_session_moves_the_schedule() {
    request::<App, _, _>(|request, _ctx| async move {
        let all: Value = request.get("/api/plasticity").await.json();
        let domains: Vec<&str> = all
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["domain"].as_str().unwrap())
            .collect();
        assert_eq!(
            domains,
            [
                "math",
                "abstraction",
                "logic",
                "music",
                "literature",
                "languages"
            ]
        );
        assert!(all
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["origin"] == "ztk" && p["level"] == 1));

        // never practised comes first, in catalog order
        let ex: Value = request.get("/api/plasticity/next").await.json();
        assert_eq!(ex["plast"], "mental-arithmetic");
        assert!(ex["seed"].as_u64().unwrap() < 1 << 53);
        let a = answer(&request, &ex, &right(&ex)).await;
        assert_eq!(a["correct"], true);
        assert_eq!(a["plast"]["streak"], 1);
        assert!(a["plast"]["due_at"].as_str().is_some());

        let ex: Value = request.get("/api/plasticity/next").await.json();
        assert_eq!(ex["plast"], "sequences", "the one just practised waits");

        // three right in a row raise the level
        let music: Value = request
            .get("/api/plasticity/next?domain=music")
            .await
            .json();
        assert_eq!(music["plast"], "intervals");
        let mut last = Value::Null;
        for _ in 0..3 {
            let ex: Value = request
                .get("/api/plasticity/next?domain=music")
                .await
                .json();
            last = answer(&request, &ex, &right(&ex)).await;
        }
        assert_eq!(last["plast"]["level"], 2);
        assert_eq!(last["plast"]["streak"], 3);

        // one slip keeps the level, a second one lowers it, and it is due again now
        let ex: Value = request
            .get("/api/plasticity/next?domain=music")
            .await
            .json();
        let a = answer(&request, &ex, "not even close").await;
        assert_eq!(
            (
                a["correct"].clone(),
                a["plast"]["level"].clone(),
                a["plast"]["streak"].clone()
            ),
            (json!(false), json!(2), json!(0))
        );
        assert!(!a["explain"].as_str().unwrap().is_empty());
        let ex: Value = request
            .get("/api/plasticity/next?domain=music")
            .await
            .json();
        let a = answer(&request, &ex, "still wrong").await;
        assert_eq!(a["plast"]["level"], 1);

        let res = request.get("/api/plasticity/next?domain=astrology").await;
        assert_eq!(res.status_code(), 400);
        let res = request
            .post("/api/plasticity/answer")
            .json(&json!({ "plast": "intervals", "seed": 1, "level": 9, "answer": "x" }))
            .await;
        assert_eq!(res.status_code(), 400, "levels run from 1 to 5");
        let res = request
            .post("/api/plasticity/answer")
            .json(&json!({ "plast": "my-own", "seed": 1, "level": 1, "answer": "x" }))
            .await;
        assert_eq!(res.status_code(), 404, "user plasts do not exist yet");
    })
    .await;
}
