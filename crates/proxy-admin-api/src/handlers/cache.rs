use crate::router::*;
pub(crate) async fn cache_status(State(s): State<ApiState>) -> Json<Value> {
    Json(json!(*s.control.cache.status.read().await))
}
pub(crate) async fn reconcile(State(s): State<ApiState>) -> ApiResult<Json<Value>> {
    s.control.reconcile().await?;
    Ok(Json(json!(*s.control.cache.status.read().await)))
}
