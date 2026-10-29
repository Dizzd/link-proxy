use crate::router::*;
pub(crate) async fn list_upstreams(State(s): State<ApiState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({"items":s.control.store.upstreams().await?})))
}
pub(crate) async fn create_upstream(
    State(s): State<ApiState>,
    Json(upstream): Json<Upstream>,
) -> ApiResult<Json<Upstream>> {
    let result = upstream.clone();
    s.control
        .update_draft_for("upstream.create", Some(&result.id), |c| {
            c.upstreams.push(upstream);
            Ok(())
        })
        .await?;
    Ok(Json(result))
}
pub(crate) async fn patch_upstream(
    State(s): State<ApiState>,
    Path(id): Path<String>,
    Json(changes): Json<Value>,
) -> ApiResult<Json<Value>> {
    s.control
        .update_draft_for("upstream.patch", Some(&id), |c| {
            patch(
                c.upstreams
                    .iter_mut()
                    .find(|u| u.id == id)
                    .ok_or_else(|| anyhow::anyhow!("upstream not found"))?,
                changes,
            )
        })
        .await?;
    Ok(Json(json!({"ok":true})))
}
pub(crate) async fn delete_upstream(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    s.control
        .update_draft_for("upstream.delete", Some(&id), |c| {
            let len = c.upstreams.len();
            c.upstreams.retain(|u| u.id != id);
            ensure!(len != c.upstreams.len(), "upstream not found");
            Ok(())
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
