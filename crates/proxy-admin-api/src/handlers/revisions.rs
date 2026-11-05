use crate::router::*;
pub(crate) async fn active(State(s): State<ApiState>) -> ApiResult<Json<Value>> {
    let snapshot = s.control.active.load_full();
    let (_, generation) = s.control.store.state().await?;
    Ok(Json(
        json!({"revision":snapshot.revision,"draft_generation":generation,"configuration":snapshot.configuration}),
    ))
}
pub(crate) async fn revisions(State(s): State<ApiState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({"items":s.control.store.revisions().await?})))
}
pub(crate) async fn preview_revision_prune(
    State(s): State<ApiState>,
    Json(input): Json<RevisionRetention>,
) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::to_value(
        s.control.preview_revision_prune(input.keep_latest).await?,
    )?))
}
pub(crate) async fn prune_revisions(
    State(s): State<ApiState>,
    Json(preview): Json<proxy_application::revision_service::RevisionPrunePreview>,
) -> ApiResult<Json<Value>> {
    let deleted_count = s.control.prune_revisions(&preview).await?;
    Ok(Json(json!({"deleted_count": deleted_count})))
}
pub(crate) async fn publish(
    State(s): State<ApiState>,
    Json(input): Json<Publish>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({"revision":s.control.publish(input.expected_revision,None).await?}),
    ))
}
pub(crate) async fn restore(
    State(s): State<ApiState>,
    Path(version): Path<i64>,
    Json(input): Json<Publish>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({"revision":s.control.publish(input.expected_revision,Some(version)).await?}),
    ))
}
pub(crate) async fn audit_events(
    State(s): State<ApiState>,
    Query(page): Query<Page>,
) -> ApiResult<Json<Value>> {
    let rows: Vec<(i64,String,i64,String,String,Option<String>)>=sqlx::query_as("SELECT id,operation,revision,created_at,actor,resource_id FROM audit_events ORDER BY id DESC LIMIT $1 OFFSET $2").bind(page.limit() as i64).bind(page.offset as i64).fetch_all(&s.control.store.pool).await?;
    Ok(Json(
        json!({"items":rows.into_iter().map(|(id,operation,revision,created_at,actor,resource_id)| AuditEvent {id,operation,revision,timestamp:created_at.clone(),created_at,actor,resource_id}).collect::<Vec<_>>()}),
    ))
}
