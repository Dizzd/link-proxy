use crate::router::*;
pub(crate) async fn list_mappings(
    State(s): State<ApiState>,
    Query(page): Query<Page>,
) -> ApiResult<Json<Value>> {
    let config = s.control.store.draft().await?;
    let found = filtered(&config, &page);
    Ok(Json(
        json!({"total":found.len(),"offset":page.offset,"items":found.iter().skip(page.offset).take(page.limit()).collect::<Vec<_>>()}),
    ))
}
pub(crate) async fn get_mapping(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Mapping>> {
    Ok(Json(
        s.control
            .store
            .draft()
            .await?
            .mappings
            .into_iter()
            .find(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("mapping not found"))?,
    ))
}
pub(crate) async fn create_mapping(
    State(s): State<ApiState>,
    Json(mapping): Json<Mapping>,
) -> ApiResult<(StatusCode, Json<Mapping>)> {
    let result = mapping.clone();
    s.control
        .update_draft_for("mapping.create", Some(&result.id), |c| {
            c.mappings.push(mapping);
            Ok(())
        })
        .await?;
    Ok((StatusCode::CREATED, Json(result)))
}
pub(crate) async fn patch_mapping(
    State(s): State<ApiState>,
    Path(id): Path<String>,
    Json(changes): Json<Value>,
) -> ApiResult<Json<Mapping>> {
    let config = s
        .control
        .update_draft_for("mapping.patch", Some(&id), |c| {
            let m = c
                .mappings
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| anyhow::anyhow!("mapping not found"))?;
            patch(m, changes)?;
            m.updated_at = now();
            Ok(())
        })
        .await?;
    Ok(Json(
        config
            .mappings
            .into_iter()
            .find(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("mapping not found"))?,
    ))
}
pub(crate) async fn delete_mapping(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    s.control
        .update_draft_for("mapping.delete", Some(&id), |c| {
            let len = c.mappings.len();
            c.mappings.retain(|m| m.id != id);
            ensure!(c.mappings.len() != len, "mapping not found");
            Ok(())
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
pub(crate) async fn validate_mapping(
    State(s): State<ApiState>,
    Json(mapping): Json<Mapping>,
) -> ApiResult<Json<Value>> {
    let mut config = s.control.store.draft().await?;
    config.mappings.retain(|m| m.id != mapping.id);
    config.mappings.push(mapping);
    s.control.compile(0, config).await?;
    Ok(Json(json!({"valid":true})))
}
pub(crate) async fn simulate(
    State(s): State<ApiState>,
    Json(input): Json<Simulation>,
) -> ApiResult<Json<Value>> {
    ensure!(
        input.path.len() <= 4096 && input.query.len() <= 8192,
        "simulation request too large"
    );
    let snapshot = if input.draft {
        Arc::new(s.control.compile(0, s.control.store.draft().await?).await?)
    } else {
        s.control.active.load_full()
    };
    let context = RequestContext {
        now: input.now.unwrap_or_else(|| SystemClock.now()),
        user_agent: &input.user_agent,
        query: &input.query,
    };
    let found = proxy_routing::simulator::simulate(
        &snapshot,
        &input.host,
        &input.path,
        &input.method,
        &context,
    )?;
    Ok(Json(
        json!({"revision":found.revision,"mapping_id":found.mapping.id,"mapping_key":found.mapping.mapping_key,"parameters":found.parameters,"action":found.action}),
    ))
}
