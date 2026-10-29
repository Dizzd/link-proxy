use crate::router::*;
pub(crate) async fn export_mappings(
    State(s): State<ApiState>,
    Query(page): Query<Page>,
) -> ApiResult<Response> {
    let mut config = s.control.store.draft().await?;
    config.mappings = filtered(&config, &page).into_iter().cloned().collect();
    let format = page.format.unwrap_or(Format::Json);
    let bytes = tokio::task::spawn_blocking(move || export(&config, format)).await??;
    Ok(download(
        format,
        &format!("mappings.{}", format.as_str()),
        bytes,
    )?)
}
