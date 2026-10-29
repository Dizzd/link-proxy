use proxy_application::ControlPlane;
use std::sync::Arc;
pub fn admin_router(control: Arc<ControlPlane>, token: &str) -> anyhow::Result<axum::Router> {
    let directory = std::path::PathBuf::from(&control.settings.admin_static_dir);
    let api = proxy_admin_api::router(control, token)?;
    let assets = tower_http::services::ServeDir::new(&directory).not_found_service(
        tower_http::services::ServeFile::new(directory.join("index.html")),
    );
    Ok(axum::Router::new().merge(api).fallback_service(assets))
}
