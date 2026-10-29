#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("mapping not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
}
