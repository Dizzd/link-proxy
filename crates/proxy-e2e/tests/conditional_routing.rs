mod common;
use common::*;
use serde_json::{Value, json};
#[tokio::test]
#[ignore = "Requires real Pingora: python3 scripts/e2e.py"]
async fn device_boolean_time_and_query_share_production_evaluator() {
    let mut m = mapping("e2e-conditions", "https://default.example/");
    m["rules"] = json!([{"id":"e2e-mobile-rule","priority":10,"condition":{"type":"all","conditions":[{"type":"device_type","in":["mobile"]},{"type":"not","condition":{"type":"query_parameter_equals","name":"skip","value":"yes"}}]},"action":{"type":"redirect","status":307,"target_url":"https://mobile.example/"}}]);
    create(m).await;
    publish().await;
    let r = client()
        .get(format!("{}/e2e-conditions", proxy()))
        .header("Host", "abc.example.com")
        .header("User-Agent", "iPhone Mobile")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 307);
    assert_eq!(r.headers()["location"], "https://mobile.example/");
    assert_eq!(
        get("/e2e-conditions?skip=yes").await.headers()["location"],
        "https://default.example/"
    );
    let before = client()
        .get(format!("{}/metrics", admin()))
        .bearer_auth(token())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let simulation=api("POST","/mappings/simulate",Some(json!({"host":"abc.example.com","path":"/e2e-conditions","user_agent":"iPhone Mobile","now":"2026-10-29T01:00:00Z"}))).await.json::<Value>().await.unwrap();
    assert_eq!(simulation["action"]["status"], 307);
    let after = client()
        .get(format!("{}/metrics", admin()))
        .bearer_auth(token())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(before, after);
}
