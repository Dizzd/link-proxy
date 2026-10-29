use crate::router::*;
pub(crate) async fn list_rules(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let mapping = get_mapping(State(s), Path(id)).await?.0;
    Ok(Json(json!({"items":mapping.rules})))
}
pub(crate) async fn create_rule(
    State(s): State<ApiState>,
    Path(id): Path<String>,
    Json(rule): Json<MappingRule>,
) -> ApiResult<Json<Value>> {
    s.control
        .update_draft_for("rule.create", Some(&id), |c| {
            let mapping = c
                .mappings
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| anyhow::anyhow!("mapping not found"))?;
            mapping.rules.push(rule);
            mapping.updated_at = now();
            Ok(())
        })
        .await?;
    Ok(Json(json!({"ok":true})))
}
pub(crate) async fn patch_rule(
    State(s): State<ApiState>,
    Path((id, rule_id)): Path<(String, String)>,
    Json(changes): Json<Value>,
) -> ApiResult<Json<Value>> {
    s.control
        .update_draft_for("rule.patch", Some(&id), |c| {
            let mapping = c
                .mappings
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| anyhow::anyhow!("mapping not found"))?;
            let rule = mapping
                .rules
                .iter_mut()
                .find(|r| r.id == rule_id)
                .ok_or_else(|| anyhow::anyhow!("rule not found"))?;
            patch(rule, changes)?;
            mapping.updated_at = now();
            Ok(())
        })
        .await?;
    Ok(Json(json!({"ok":true})))
}
pub(crate) async fn delete_rule(
    State(s): State<ApiState>,
    Path((id, rule_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    s.control
        .update_draft_for("rule.delete", Some(&id), |c| {
            let mapping = c
                .mappings
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| anyhow::anyhow!("mapping not found"))?;
            let len = mapping.rules.len();
            mapping.rules.retain(|r| r.id != rule_id);
            ensure!(mapping.rules.len() != len, "rule not found");
            mapping.updated_at = now();
            Ok(())
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
