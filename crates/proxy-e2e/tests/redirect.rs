mod common;
use common::*;
use serde_json::json;
#[tokio::test]
#[ignore = "Requires real Pingora and Axum listeners: python3 scripts/e2e.py"]
async fn static_redirect_authentication_hosts_and_methods() {
    assert_eq!(
        client()
            .get(format!("{}/api/v1/mappings", admin()))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    create(mapping("e2e-redirect", "https://destination.example/start")).await;
    publish().await;
    let r = get("/e2e-redirect").await;
    assert_eq!(r.status(), 302);
    assert_eq!(r.headers()["location"], "https://destination.example/start");
    assert_eq!(
        client()
            .get(format!("{}/e2e-redirect", proxy()))
            .header("Host", "second.example.com")
            .send()
            .await
            .unwrap()
            .status(),
        302
    );
    assert_eq!(
        client()
            .post(format!("{}/e2e-redirect", proxy()))
            .header("Host", "abc.example.com")
            .send()
            .await
            .unwrap()
            .status(),
        405
    );
    assert_eq!(get("/missing").await.status(), 404);
    let mut invalid = mapping("bad-destination", "file:///etc/passwd");
    assert_eq!(
        api("POST", "/mappings", Some(invalid.clone()))
            .await
            .status(),
        422
    );
    invalid["default_action"] =
        json!({"type":"redirect","status":302,"target_url":"https://example.com/\r\nInjected:yes"});
    // Literal backslash URLs are normalized by url; direct CRLF is always rejected.
    invalid["default_action"]["target_url"] = json!("https://example.com/\r\nInjected:yes");
    assert_eq!(api("POST", "/mappings", Some(invalid)).await.status(), 422);
}
