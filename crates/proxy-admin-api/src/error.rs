use crate::router::*;
pub struct ApiError(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}
pub(crate) type ApiResult<T> = std::result::Result<T, ApiError>;
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let message = self.0.to_string();
        let status = if let Some(error) = self.0.downcast_ref::<crate::auth::AuthError>() {
            error.status()
        } else if self
            .0
            .downcast_ref::<proxy_routing::error::RoutingError>()
            .is_some_and(|e| matches!(e, proxy_routing::error::RoutingError::MethodNotAllowed))
        {
            StatusCode::METHOD_NOT_ALLOWED
        } else if self
            .0
            .downcast_ref::<sqlx::Error>()
            .is_some_and(|e| matches!(e, sqlx::Error::RowNotFound))
            || message.contains("not found")
        {
            StatusCode::NOT_FOUND
        } else if message.contains("conflict")
            || message.contains("changed since validation")
            || message.contains("existing mapping")
        {
            StatusCode::CONFLICT
        } else if message.contains("capacity") || message.contains("too many staged") {
            StatusCode::TOO_MANY_REQUESTS
        } else if self.0.downcast_ref::<sqlx::Error>().is_some() {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::UNPROCESSABLE_ENTITY
        };
        if status.is_server_error() {
            tracing::error!(error = %self.0,"administration operation failed");
        }
        (status,Json(json!({"error":{"code":status.as_str(),"message":if status.is_server_error() { "Database operation failed" } else { &message }}}))).into_response()
    }
}
