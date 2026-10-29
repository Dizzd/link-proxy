use crate::service::*;
pub(crate) fn prepare(
    session: &mut Session,
    request: &mut RequestHeader,
    ctx: &mut ProxyContext,
) -> Result<()> {
    let Some(Action::ReverseProxy {
        upstream_id,
        rewrite,
        preserve_host,
    }) = &ctx.action
    else {
        return Err(proxy_error("Proxy action missing"));
    };
    let upstream = ctx
        .snapshot
        .as_ref()
        .and_then(|s| s.upstreams.get(upstream_id))
        .ok_or_else(|| proxy_error("Upstream missing"))?;
    if let Some(template) = rewrite {
        let path =
            rewrite_path(template, &ctx.parameters).map_err(|_| bad_request("Invalid rewrite"))?;
        let uri = match request.uri.query() {
            Some(query) => format!("{path}?{query}"),
            None => path,
        };
        request.set_uri(
            uri.parse()
                .map_err(|_| bad_request("Invalid rewrite URI"))?,
        );
    }
    // Pingora strips hop-by-hop and Connection-nominated fields. Replace all untrusted forwarding metadata.
    let forwarded: Vec<_> = request
        .headers
        .keys()
        .filter(|h| h.as_str().starts_with("x-forwarded-") || h.as_str() == "forwarded")
        .cloned()
        .collect();
    for name in forwarded {
        request.remove_header(&name);
    }
    request.insert_header("X-Forwarded-Host", ctx.host.clone())?;
    request.insert_header("X-Forwarded-Proto", "http")?;
    if let Some(ip) = session
        .client_addr()
        .and_then(|a| a.as_inet())
        .map(|a| a.ip())
    {
        request.insert_header("X-Forwarded-For", ip.to_string())?;
    }
    if !preserve_host {
        let host = if (upstream.scheme == "https" && upstream.port == 443)
            || (upstream.scheme == "http" && upstream.port == 80)
        {
            upstream.hostname.clone()
        } else {
            format!("{}:{}", upstream.hostname, upstream.port)
        };
        request.insert_header("Host", host)?;
    }
    Ok(())
}
