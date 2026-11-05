mod common;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use common::*;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
const TOKEN: &str = "test-token-with-at-least-24-bytes";
async fn request(app: &Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("Authorization", format!("Bearer {TOKEN}"))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
#[tokio::test]
async fn administration_requires_authentication() {
    let app = proxy_admin_api::router(control().await, TOKEN).unwrap();
    for path in [
        "/api/v1/mappings",
        "/api/v1/system/cache/status",
        "/api/v1/imports",
        "/metrics",
        "/health",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn revision_cleanup_requires_preview_validates_retention_and_rejects_stale_confirmation() {
    let c = control().await;
    let app = proxy_admin_api::router(c.clone(), TOKEN).unwrap();
    let path = "/api/v1/config/revisions/prune/preview";
    let (status, empty) = request(&app, "POST", path, json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(empty["keep_latest"], 100);
    assert_eq!(empty["versions"], json!([]));
    for keep in [0, 10001] {
        assert_eq!(
            request(&app, "POST", path, json!({"keep_latest":keep}))
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        request(&app, "POST", "/api/v1/config/revisions/prune", json!({}))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for expected in 0..3 {
        c.publish(expected, None).await.unwrap();
    }
    let (_, preview) = request(&app, "POST", path, json!({"keep_latest":1})).await;
    assert_eq!(preview["versions"], json!([1, 2]));
    c.publish(3, None).await.unwrap();
    assert_eq!(
        request(&app, "POST", "/api/v1/config/revisions/prune", preview)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let (_, preview) = request(&app, "POST", path, json!({"keep_latest":1})).await;
    let (status, result) = request(&app, "POST", "/api/v1/config/revisions/prune", preview).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["deleted_count"], 3);
    assert_eq!(c.active.load().revision, 4);
    let (status, _) = request(
        &app,
        "POST",
        "/api/v1/config/revisions/prune",
        json!({"padding":"x".repeat(16 * 1024)}),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn revision_cleanup_endpoints_require_authentication() {
    let app = proxy_admin_api::router(control().await, TOKEN).unwrap();
    for path in [
        "/api/v1/config/revisions/prune/preview",
        "/api/v1/config/revisions/prune",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
#[tokio::test]
async fn mapping_crud_publish_simulate_and_revision_conflict() {
    let c = control().await;
    let app = proxy_admin_api::router(c.clone(), TOKEN).unwrap();
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/mappings",
            serde_json::to_value(mapping("a", "/a")).unwrap()
        )
        .await
        .0,
        StatusCode::CREATED
    );
    assert_eq!(
        request(&app, "GET", "/api/v1/mappings?search=a", Value::Null)
            .await
            .1["total"],
        1
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/config/publish",
            json!({"expected_revision":0})
        )
        .await
        .1["revision"],
        1
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/mappings/simulate",
            json!({"host":"example.com","path":"/a"})
        )
        .await
        .1["action"]["type"],
        "redirect"
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/config/publish",
            json!({"expected_revision":0})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &app,
            "PATCH",
            "/api/v1/mappings/id-a",
            json!({"enabled":false})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert!(c.active.load().lookup("example.com", "/a", "GET").is_some());
    assert_eq!(
        request(&app, "DELETE", "/api/v1/mappings/id-a", Value::Null)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
}
#[tokio::test]
async fn templates_and_exports_are_authenticated_downloads() {
    let app = proxy_admin_api::router(control().await, TOKEN).unwrap();
    for format in ["csv", "json", "xlsx"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/imports/templates?format={format}"))
                    .header("Authorization", format!("Bearer {TOKEN}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("content-disposition"));
    }
}
#[tokio::test]
async fn ordinary_admin_body_limit_is_enforced() {
    let app = proxy_admin_api::router(control().await, TOKEN).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/mappings")
                .header("Authorization", format!("Bearer {TOKEN}"))
                .header("Content-Type", "application/json")
                .body(Body::from("x".repeat(1024 * 1024 + 1)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn malformed_json_is_structured_and_readiness_checks_proxy_listener() {
    let c = control().await;
    let app = proxy_admin_api::router(c, TOKEN).unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/mappings")
                .header("Authorization", format!("Bearer {TOKEN}"))
                .header("Content-Type", "application/json")
                .body(Body::from("{"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(body["error"]["message"].is_string());
    let mut settings = proxy_cache::config::Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    };
    settings.proxy_listen = "127.0.0.1:0".into();
    let c = proxy_application::ControlPlane::open(settings)
        .await
        .unwrap();
    let app = proxy_admin_api::router(c, TOKEN).unwrap();
    assert_eq!(
        request(&app, "GET", "/health/ready", Value::Null).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        request(&app, "GET", "/health/live", Value::Null).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn audit_retains_authenticated_actor_and_edited_resource() {
    let app = proxy_admin_api::router(control().await, TOKEN).unwrap();
    let m = mapping("audited", "/audited");
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/mappings",
            serde_json::to_value(m).unwrap()
        )
        .await
        .0,
        StatusCode::CREATED
    );
    let (_, body) = request(&app, "GET", "/api/v1/audit-events", Value::Null).await;
    assert_eq!(body["items"][0]["operation"], "mapping.create");
    assert_eq!(body["items"][0]["resource_id"], "id-audited");
    assert_eq!(body["items"][0]["actor"], "admin-token");
    assert!(body["items"][0]["timestamp"].is_string());
}

#[tokio::test]
async fn upload_capacity_is_reserved_before_multipart_file_buffering() {
    let c = control().await;
    let _slots = c.upload_slots.clone().acquire_many_owned(2).await.unwrap();
    let app = proxy_admin_api::router(c, TOKEN).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/imports")
                .header("Authorization", format!("Bearer {TOKEN}"))
                .header(
                    "Content-Type",
                    "multipart/form-data; boundary=bounded-upload",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}
