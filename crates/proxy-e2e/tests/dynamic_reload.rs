mod common;
use common::*;
use serde_json::json;
#[tokio::test]
#[ignore = "Requires real Pingora: python3 scripts/e2e.py"]
async fn publish_updates_without_restart_and_restore_creates_revision() {
    create(mapping("e2e-dynamic", "https://original.example/")).await;
    let first = publish().await;
    assert!(api("PATCH","/mappings/e2e-dynamic",Some(json!({"default_action":{"type":"redirect","status":302,"target_url":"https://changed.example/"}}))).await.status().is_success());
    assert_eq!(
        get("/e2e-dynamic").await.headers()["location"],
        "https://original.example/"
    );
    let second = publish().await;
    assert_eq!(
        get("/e2e-dynamic").await.headers()["location"],
        "https://changed.example/"
    );
    assert_eq!(
        api(
            "POST",
            "/config/publish",
            Some(json!({"expected_revision":first}))
        )
        .await
        .status(),
        409
    );
    let r = api(
        "POST",
        &format!("/config/revisions/{first}/restore"),
        Some(json!({"expected_revision":second})),
    )
    .await;
    assert!(r.status().is_success());
    assert_eq!(revision().await, second + 1);
    assert_eq!(
        get("/e2e-dynamic").await.headers()["location"],
        "https://original.example/"
    );
}
