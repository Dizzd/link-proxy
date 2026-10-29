use anyhow::{Result, ensure};
use proxy_domain::*;
use std::collections::HashMap;
pub fn normalize_host(input: &str) -> Result<String> {
    ensure!(
        input.len() <= 253
            && !input.is_empty()
            && !input.contains(['/', '\\', '@', ' ', '\r', '\n', '?', '#']),
        "invalid hostname"
    );
    let authority: http::uri::Authority = input.parse()?;
    let host = authority.host().trim_end_matches('.').to_ascii_lowercase();
    ensure!(!host.is_empty(), "empty hostname");
    if !host.starts_with('[') {
        ensure!(
            host.split('.').all(|l| !l.is_empty()
                && l.len() <= 63
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')),
            "invalid hostname labels"
        );
    }
    Ok(host)
}
pub fn validate_path(path: &str) -> Result<()> {
    ensure!(
        path.starts_with('/')
            && path.len() <= 4096
            && !path.contains(['?', '#', '\\'])
            && !path.chars().any(char::is_control),
        "invalid source path"
    );
    ensure!(
        !path.split('/').any(|p| p == "." || p == ".."),
        "dot segments are not accepted"
    );
    Ok(())
}
pub fn validate_action(
    action: &Action,
    upstreams: &HashMap<String, Upstream>,
    allowlist: &[String],
) -> Result<()> {
    match action {
        Action::Redirect {
            status, target_url, ..
        } => {
            ensure!(
                [301, 302, 303, 307, 308].contains(status),
                "unsupported redirect status"
            );
            ensure!(
                target_url.len() <= 8192 && !target_url.chars().any(char::is_control),
                "unsafe redirect URL"
            );
            let url = url::Url::parse(target_url)?;
            ensure!(
                matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none(),
                "redirect requires HTTP(S) URL without credentials"
            );
            ensure!(
                allowlist.is_empty()
                    || allowlist.iter().any(|h| Some(h.as_str()) == url.host_str()),
                "redirect host is outside allowlist"
            );
            http::HeaderValue::from_str(target_url)?;
        }
        Action::ReverseProxy {
            upstream_id,
            rewrite,
            ..
        } => {
            ensure!(
                upstreams.contains_key(upstream_id),
                "unknown upstream: {upstream_id}"
            );
            if let Some(rewrite) = rewrite {
                validate_path(rewrite)?;
            }
        }
    }
    Ok(())
}
