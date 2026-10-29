pub fn request_complete(
    revision: Option<i64>,
    status: Option<u16>,
    duration_seconds: f64,
    failed: bool,
) {
    ::tracing::info!(
        revision,
        status,
        duration_seconds,
        failed,
        "proxy request completed"
    );
}
