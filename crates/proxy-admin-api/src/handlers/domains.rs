use crate::router::*;
pub(crate) async fn list_domains(State(s): State<ApiState>) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({"items":s.control.store.draft().await?.domains}),
    ))
}
pub(crate) async fn create_domain(
    State(s): State<ApiState>,
    Json(domain): Json<Domain>,
) -> ApiResult<Json<Domain>> {
    let result = domain.clone();
    s.control
        .update_draft_for("domain.create", Some(&result.id), |c| {
            c.domains.push(domain);
            Ok(())
        })
        .await?;
    Ok(Json(result))
}
pub(crate) async fn patch_domain(
    State(s): State<ApiState>,
    Path(id): Path<String>,
    Json(changes): Json<Value>,
) -> ApiResult<Json<Value>> {
    s.control
        .update_draft_for("domain.patch", Some(&id), |c| {
            patch(
                c.domains
                    .iter_mut()
                    .find(|d| d.id == id)
                    .ok_or_else(|| anyhow::anyhow!("domain not found"))?,
                changes,
            )
        })
        .await?;
    Ok(Json(json!({"ok":true})))
}
pub(crate) async fn delete_domain(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    s.control
        .update_draft_for("domain.delete", Some(&id), |c| {
            let len = c.domains.len();
            c.domains.retain(|d| d.id != id);
            ensure!(len != c.domains.len(), "domain not found");
            Ok(())
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
