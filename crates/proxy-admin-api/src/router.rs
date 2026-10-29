pub(crate) use anyhow::Result;
pub(crate) use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
pub(crate) use proxy_application::ControlPlane;
pub(crate) use proxy_domain::*;
pub(crate) use proxy_import::{
    ConflictPolicy, Format, ImportJobError, export, spreadsheet_safe, template,
};
pub(crate) use proxy_routing::{
    RoutingSnapshot,
    conditions::{Clock, RequestContext, SystemClock},
};
pub(crate) use serde::Deserialize;
pub(crate) use serde_json::{Value, json};
pub(crate) use sha2::{Digest, Sha256};
pub(crate) use std::{sync::Arc, time::Instant};
pub(crate) use subtle::ConstantTimeEq;
pub(crate) use tokio::sync::Mutex;

pub fn router(control: Arc<ControlPlane>, token: &str) -> Result<Router> {
    ensure!(
        token.is_empty() || token.len() >= 24,
        "admin API token must have at least 24 bytes"
    );
    let max_body = control
        .settings
        .imports
        .max_upload_bytes
        .saturating_add(64 * 1024);
    let state = ApiState {
        control,
        token_hash: if token.is_empty() {
            None
        } else {
            Some(Sha256::digest(token.as_bytes()).into())
        },
        login_rate: Arc::new(Mutex::new((Instant::now(), 0))),
        auth_workers: Arc::new(tokio::sync::Semaphore::new(2)),
        rate: Arc::new(Mutex::new((Instant::now(), 0))),
    };
    let api = Router::new()
        .route("/auth/session", get(crate::handlers::auth::session))
        .route("/auth/logout", post(crate::handlers::auth::logout))
        .route(
            "/auth/password",
            post(crate::handlers::auth::change_password),
        )
        .route("/system/database/status", get(database_status))
        .route("/mappings", get(list_mappings).post(create_mapping))
        .route("/mappings/validate", post(validate_mapping))
        .route("/mappings/simulate", post(simulate))
        .route(
            "/mappings/{id}",
            get(get_mapping).patch(patch_mapping).delete(delete_mapping),
        )
        .route("/mappings/{id}/rules", get(list_rules).post(create_rule))
        .route(
            "/mappings/{id}/rules/{rule_id}",
            axum::routing::patch(patch_rule).delete(delete_rule),
        )
        .route("/upstreams", get(list_upstreams).post(create_upstream))
        .route(
            "/upstreams/{id}",
            axum::routing::patch(patch_upstream).delete(delete_upstream),
        )
        .route("/domains", get(list_domains).post(create_domain))
        .route(
            "/domains/{id}",
            axum::routing::patch(patch_domain).delete(delete_domain),
        )
        .route("/config/active", get(active))
        .route("/config/revisions", get(revisions))
        .route("/config/publish", post(publish))
        .route("/config/revisions/{version}/restore", post(restore))
        .route("/system/cache/status", get(cache_status))
        .route("/system/cache/reconcile", post(reconcile))
        .route("/imports", get(list_imports).post(upload))
        .route("/imports/templates", get(import_template))
        .route("/imports/{id}", get(get_import))
        .route("/imports/{id}/preview", get(preview_import))
        .route("/imports/{id}/errors", get(import_errors))
        .route("/imports/{id}/commit", post(commit_import))
        .route("/imports/{id}/cancel", post(cancel_import))
        .route("/exports/mappings", get(export_mappings))
        .route("/audit-events", get(audit_events));
    let protected = Router::new()
        .nest("/api/v1", api)
        .route("/metrics", get(metrics))
        .route("/health", get(health))
        .layer(DefaultBodyLimit::max(max_body))
        .layer(middleware::from_fn_with_state(state.clone(), authorize))
        .with_state(state.clone());
    let public = Router::new()
        .route("/api/v1/auth/login", post(crate::handlers::auth::login))
        .route("/health/live", get(health))
        .route("/health/ready", get(ready))
        .layer(DefaultBodyLimit::max(4096))
        .layer(middleware::map_response(crate::auth::structured))
        .with_state(state);
    Ok(Router::new().merge(protected).merge(public))
}
pub(crate) fn filtered<'a>(config: &'a Configuration, page: &Page) -> Vec<&'a Mapping> {
    config
        .mappings
        .iter()
        .filter(|m| {
            page.search
                .as_ref()
                .is_none_or(|q| m.mapping_key.contains(q) || m.name.contains(q))
        })
        .filter(|m| {
            page.domain
                .as_ref()
                .is_none_or(|d| m.source_matcher.hosts.contains(d))
        })
        .filter(|m| page.enabled.is_none_or(|e| m.enabled == e))
        .filter(|m| {
            page.ids
                .as_ref()
                .is_none_or(|ids| ids.split(',').any(|id| id == m.id))
        })
        .filter(|m| {
            page.action.as_ref().is_none_or(|a| {
                matches!(
                    (&m.default_action, a.as_str()),
                    (Action::Redirect { .. }, "redirect")
                        | (Action::ReverseProxy { .. }, "reverse_proxy")
                )
            })
        })
        .collect()
}
pub(crate) fn patch<T: serde::Serialize + serde::de::DeserializeOwned>(
    current: &mut T,
    changes: Value,
) -> Result<()> {
    let mut value = serde_json::to_value(&*current)?;
    let changes = changes
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("PATCH must be an object"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("invalid PATCH target"))?;
    for (key, change) in changes {
        ensure!(
            !matches!(key.as_str(), "id" | "created_at"),
            "identity fields are immutable"
        );
        ensure!(object.contains_key(key), "unknown field: {key}");
        object.insert(key.clone(), change.clone());
    }
    *current = serde_json::from_value(value)?;
    Ok(())
}
pub(crate) fn download(format: Format, name: &str, bytes: Vec<u8>) -> Result<Response> {
    Ok((
        [
            (header::CONTENT_TYPE, format.content_type()),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{name}\""),
            ),
        ],
        bytes,
    )
        .into_response())
}

/// Compile a supplied complete configuration without changing persistent or active state.
pub fn validate_configuration(config: Configuration) -> Result<()> {
    RoutingSnapshot::compile(0, config, &[])?;
    Ok(())
}

pub(crate) use crate::handlers::mappings::*;

pub(crate) use crate::handlers::rules::*;

pub(crate) use crate::handlers::upstreams::*;

pub(crate) use crate::handlers::domains::*;

pub(crate) use crate::handlers::imports::*;

pub(crate) use crate::handlers::exports::*;

pub(crate) use crate::handlers::revisions::*;

pub(crate) use crate::handlers::cache::*;

pub(crate) use crate::handlers::operations::*;

pub(crate) use crate::auth::authorize;
pub(crate) use crate::dto::{import::*, mapping::*, response::*};
pub(crate) use crate::error::*;
pub(crate) use crate::state::*;

async fn database_status(State(s): State<ApiState>) -> ApiResult<Json<Value>> {
    let (revision, _) = s.control.store.state().await?;
    Ok(Json(
        json!({"backend":s.control.store.backend.as_str(),"connected":true,"database_revision":revision,"active_revision":s.control.active.load().revision}),
    ))
}
