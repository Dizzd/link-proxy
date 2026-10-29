mod common;
use common::*;
use serde_json::{Value, json};
#[tokio::test]
#[ignore = "Requires real Pingora and echo upstream: python3 scripts/e2e.py"]
async fn streaming_proxy_rewrite_host_and_forwarding_policy() {
    let port: u16 = std::env::var("PROXY_E2E_UPSTREAM_PORT")
        .unwrap()
        .parse()
        .unwrap();
    let r=api("POST","/upstreams",Some(json!({"id":"e2e-backend","name":"echo","scheme":"http","hostname":"127.0.0.1","port":port}))).await;
    assert!(r.status().is_success());
    let mut m = mapping("e2e-proxy", "https://unused.example/");
    m["source_matcher"]["path_type"] = json!("template");
    m["source_matcher"]["path_value"] = json!("/echo/{id}");
    m["source_matcher"]["methods"] = json!(["GET", "POST"]);
    m["default_action"] = json!({"type":"reverse_proxy","upstream_id":"e2e-backend","rewrite":"/received/{id}","preserve_host":false});
    create(m).await;
    publish().await;
    let r = client()
        .post(format!("{}/echo/123?x=1", proxy()))
        .header("Host", "abc.example.com")
        .header("X-Forwarded-For", "spoofed")
        .body("streamed-body")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["path"], "/received/123?x=1");
    assert_eq!(body["body"], "streamed-body");
    assert_eq!(body["host"], format!("127.0.0.1:{port}"));
    assert_ne!(body["forwarded_for"], "spoofed");
    let shared = client()
        .get(format!("{}/echo/456", proxy()))
        .header("Host", "second.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(shared.status(), 200);
    assert_eq!(
        shared.json::<Value>().await.unwrap()["path"],
        "/received/456"
    );
    // A private address outside the explicit loopback allowlist must never be connected.
    assert!(api("POST","/upstreams",Some(json!({"id":"e2e-denied","name":"denied","scheme":"http","hostname":"169.254.169.254","port":80}))).await.status().is_success());
    let mut denied = mapping("e2e-denied", "https://unused.example/");
    denied["default_action"] = json!({"type":"reverse_proxy","upstream_id":"e2e-denied"});
    create(denied).await;
    publish().await;
    assert_eq!(get("/e2e-denied").await.status(), 502);
}
