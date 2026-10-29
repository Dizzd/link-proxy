use super::*;
pub fn redirect_location(
    action: &Action,
    host: &str,
    path: &str,
    query: Option<&str>,
) -> Result<String> {
    let Action::Redirect {
        target_url,
        path_handling,
        query_policy,
        ..
    } = action
    else {
        anyhow::bail!("not a redirect")
    };
    let mut target = url::Url::parse(target_url)?;
    if matches!(path_handling, PathHandling::Append) {
        target.set_path(&format!(
            "{}/{}",
            target.path().trim_end_matches('/'),
            path.trim_start_matches('/')
        ));
    }
    match query_policy {
        QueryPolicy::Drop => {}
        QueryPolicy::Preserve => target.set_query(query),
        QueryPolicy::Merge => {
            if let Some(query) = query {
                let merged = match target.query() {
                    Some(q) => format!("{q}&{query}"),
                    None => query.into(),
                };
                target.set_query(Some(&merged));
            }
        }
    }
    ensure!(
        !(normalize_host(target.host_str().unwrap_or_default())? == normalize_host(host)?
            && target.path() == path
            && target.port_or_known_default()
                == host
                    .parse::<http::uri::Authority>()
                    .ok()
                    .and_then(|a| a.port_u16())
                    .or(Some(80))),
        "direct redirect loop"
    );
    let location = target.to_string();
    http::HeaderValue::from_str(&location)?;
    Ok(location)
}
