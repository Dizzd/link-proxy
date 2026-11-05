use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use proxy_application::ControlPlane;
use proxy_cache::config::Settings;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;
const PASSWORD: &str = "test-only-valid-password-123";
const NEW_PASSWORD: &str = "test-only-changed-password-456";
async fn add_user(control: &ControlPlane) {
    let salt = argon2::password_hash::SaltString::generate(&mut rand_core::OsRng);
    let hash = argon2::PasswordHasher::hash_password(
        &argon2::Argon2::default(),
        PASSWORD.as_bytes(),
        &salt,
    )
    .unwrap()
    .to_string();
    assert!(
        control
            .store
            .bootstrap_admin("auth-user", "admin", &hash)
            .await
            .unwrap()
    );
}
async fn setup() -> (Arc<ControlPlane>, Router) {
    let c = ControlPlane::open(Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    })
    .await
    .unwrap();
    add_user(&c).await;
    let app = proxy_admin_api::router(c.clone(), "").unwrap();
    (c, app)
}
async fn call(
    app: &Router,
    method: &str,
    path: &str,
    body: Value,
    cookie: Option<&str>,
    csrf: Option<&str>,
    origin: Option<&str>,
) -> (StatusCode, Value, Option<String>) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("Host", "localhost")
        .header("Content-Type", "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header("Cookie", cookie)
    }
    if let Some(csrf) = csrf {
        builder = builder.header("X-CSRF-Token", csrf)
    }
    if let Some(origin) = origin {
        builder = builder.header("Origin", origin)
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let cookie = response
        .headers()
        .get("set-cookie")
        .map(|value| value.to_str().unwrap().to_owned());
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&body).unwrap_or(Value::Null),
        cookie,
    )
}
async fn login(app: &Router, password: &str) -> (String, String) {
    let (status, body, cookie) = call(
        app,
        "POST",
        "/api/v1/auth/login",
        json!({"username":"Admin","password":password}),
        None,
        None,
        Some("http://localhost"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let cookie = cookie.unwrap();
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict"));
    (
        cookie.split(';').next().unwrap().to_owned(),
        body["csrf_token"].as_str().unwrap().to_owned(),
    )
}
#[tokio::test]
async fn login_stores_argon2_password_and_session_hash_and_restores_session() {
    let (c, app) = setup().await;
    let (cookie, _) = login(&app, PASSWORD).await;
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM admin_users")
        .fetch_one(&c.store.pool)
        .await
        .unwrap();
    assert!(hash.starts_with("$argon2id$"));
    assert!(!hash.contains(PASSWORD));
    let stored: String = sqlx::query_scalar("SELECT id FROM admin_sessions")
        .fetch_one(&c.store.pool)
        .await
        .unwrap();
    assert_ne!(stored, cookie.split('=').nth(1).unwrap());
    assert_eq!(stored.len(), 64);
    let (status, body, _) = call(
        &app,
        "GET",
        "/api/v1/auth/session",
        Value::Null,
        Some(&cookie),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["username"], "admin");
    assert!(body.get("password_hash").is_none());
}

#[tokio::test]
async fn revision_cleanup_preview_and_commit_require_session_csrf() {
    let (_, app) = setup().await;
    let (cookie, csrf) = login(&app, PASSWORD).await;
    for path in [
        "/api/v1/config/revisions/prune/preview",
        "/api/v1/config/revisions/prune",
    ] {
        assert_eq!(
            call(
                &app,
                "POST",
                path,
                json!({}),
                Some(&cookie),
                None,
                Some("http://localhost")
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            call(
                &app,
                "POST",
                path,
                json!({}),
                Some(&cookie),
                Some(&csrf),
                Some("http://other.example")
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    let (status, preview, _) = call(
        &app,
        "POST",
        "/api/v1/config/revisions/prune/preview",
        json!({}),
        Some(&cookie),
        Some(&csrf),
        Some("http://localhost"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, result, _) = call(
        &app,
        "POST",
        "/api/v1/config/revisions/prune",
        preview,
        Some(&cookie),
        Some(&csrf),
        Some("http://localhost"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["deleted_count"], 0);
}
#[tokio::test]
async fn invalid_credentials_are_generic_and_token_auth_is_disabled_when_unconfigured() {
    let (_, app) = setup().await;
    let a = call(
        &app,
        "POST",
        "/api/v1/auth/login",
        json!({"username":"admin","password":"wrong"}),
        None,
        None,
        None,
    )
    .await;
    let b = call(
        &app,
        "POST",
        "/api/v1/auth/login",
        json!({"username":"missing-user","password":"wrong"}),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(a.0, StatusCode::UNAUTHORIZED);
    assert_eq!(a.1, b.1);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/mappings")
                .header("Authorization", "Bearer test-token-with-at-least-24-bytes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
#[tokio::test]
async fn csrf_origin_audit_actor_and_server_logout_are_enforced() {
    let (_, app) = setup().await;
    let (cookie, csrf) = login(&app, PASSWORD).await;
    let mapping = json!({"id":"session-map","mapping_key":"session-map","source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":"/session-map"},"default_action":{"type":"redirect","status":302,"target_url":"https://target.example/"}});
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/mappings",
            mapping.clone(),
            Some(&cookie),
            None,
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/mappings",
            mapping.clone(),
            Some(&cookie),
            Some(&csrf),
            Some("https://evil.example")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/mappings",
            mapping,
            Some(&cookie),
            Some(&csrf),
            Some("http://localhost")
        )
        .await
        .0,
        StatusCode::CREATED
    );
    let (_, audit, _) = call(
        &app,
        "GET",
        "/api/v1/audit-events",
        Value::Null,
        Some(&cookie),
        None,
        None,
    )
    .await;
    assert_eq!(audit["items"][0]["actor"], "admin");
    assert_eq!(audit["items"][0]["operation"], "mapping.create");
    let (status, _, deleted) = call(
        &app,
        "POST",
        "/api/v1/auth/logout",
        Value::Null,
        Some(&cookie),
        Some(&csrf),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(deleted.unwrap().contains("Max-Age=0"));
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/auth/session",
            Value::Null,
            Some(&cookie),
            None,
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn password_change_revokes_all_sessions_and_old_password() {
    let (_, app) = setup().await;
    let (first, csrf) = login(&app, PASSWORD).await;
    let (second, _) = login(&app, PASSWORD).await;
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/auth/password",
            json!({"current_password":"wrong","new_password":NEW_PASSWORD}),
            Some(&first),
            Some(&csrf),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/auth/password",
            json!({"current_password":PASSWORD,"new_password":NEW_PASSWORD}),
            Some(&first),
            Some(&csrf),
            None
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    for cookie in [&first, &second] {
        assert_eq!(
            call(
                &app,
                "GET",
                "/api/v1/auth/session",
                Value::Null,
                Some(cookie),
                None,
                None
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        )
    }
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/auth/login",
            json!({"username":"admin","password":PASSWORD}),
            None,
            None,
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    login(&app, NEW_PASSWORD).await;
}
#[tokio::test]
async fn expired_sessions_disabled_users_and_cross_origin_login_are_rejected() {
    let (c, app) = setup().await;
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/auth/login",
            json!({"username":"admin","password":PASSWORD}),
            None,
            None,
            Some("https://evil.example")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (cookie, _) = login(&app, PASSWORD).await;
    sqlx::query("UPDATE admin_sessions SET expires_at=0")
        .execute(&c.store.pool)
        .await
        .unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/auth/session",
            Value::Null,
            Some(&cookie),
            None,
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (cookie, _) = login(&app, PASSWORD).await;
    sqlx::query("UPDATE admin_users SET enabled=0")
        .execute(&c.store.pool)
        .await
        .unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/auth/session",
            Value::Null,
            Some(&cookie),
            None,
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn login_attempts_are_rate_limited() {
    let settings = Settings {
        database_url: "sqlite::memory:".into(),
        auth: proxy_cache::config::AuthSettings {
            login_requests_per_minute: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let c = ControlPlane::open(settings).await.unwrap();
    add_user(&c).await;
    let app = proxy_admin_api::router(c, "").unwrap();
    let bad = json!({"username":"admin","password":"wrong"});
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/auth/login",
            bad.clone(),
            None,
            None,
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&app, "POST", "/api/v1/auth/login", bad, None, None, None)
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS
    );
}
#[tokio::test]
async fn secure_cookie_option_and_malformed_login_limits_are_enforced() {
    let c = ControlPlane::open(Settings {
        database_url: "sqlite::memory:".into(),
        auth: proxy_cache::config::AuthSettings {
            secure_cookies: true,
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .unwrap();
    add_user(&c).await;
    let app = proxy_admin_api::router(c, "").unwrap();
    let (status, _, cookie) = call(
        &app,
        "POST",
        "/api/v1/auth/login",
        json!({"username":"admin","password":PASSWORD}),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(cookie.unwrap().contains("; Secure"));
    for (body, expected) in [
        ("{".to_owned(), StatusCode::BAD_REQUEST),
        ("x".repeat(8192), StatusCode::PAYLOAD_TOO_LARGE),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let error: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert!(error["error"]["message"].is_string());
    }
}
#[tokio::test]
async fn sessions_and_users_survive_restart_and_bootstrap_never_overwrites() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let settings = Settings {
        database_url: format!("sqlite://{}", file.path().display()),
        ..Default::default()
    };
    let c = ControlPlane::open(settings.clone()).await.unwrap();
    add_user(&c).await;
    let app = proxy_admin_api::router(c.clone(), "").unwrap();
    let (cookie, _) = login(&app, PASSWORD).await;
    assert!(
        !c.store
            .bootstrap_admin("another-user", "other-admin", "invalid-replacement")
            .await
            .unwrap()
    );
    drop(app);
    c.store.pool.close().await;
    drop(c);
    let c = ControlPlane::open(settings).await.unwrap();
    let app = proxy_admin_api::router(c, "").unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/auth/session",
            Value::Null,
            Some(&cookie),
            None,
            None
        )
        .await
        .0,
        StatusCode::OK
    );
    login(&app, PASSWORD).await;
}
