use crate::service::*;
pub(crate) async fn select(control: &ControlPlane, ctx: &ProxyContext) -> Result<Box<HttpPeer>> {
    let Some(Action::ReverseProxy { upstream_id, .. }) = &ctx.action else {
        return Err(proxy_error("Proxy action missing"));
    };
    let upstream = ctx
        .snapshot
        .as_ref()
        .and_then(|s| s.upstreams.get(upstream_id))
        .ok_or_else(|| proxy_error("Upstream missing"))?;
    let addresses = tokio::time::timeout(
        Duration::from_millis(upstream.connect_timeout_ms),
        tokio::net::lookup_host((upstream.hostname.trim_matches(['[', ']']), upstream.port)),
    )
    .await
    .map_err(|_| proxy_error("DNS timeout"))?
    .map_err(|_| proxy_error("DNS failure"))?;
    // Pass a validated concrete SocketAddr to Pingora; it must never resolve the hostname again.
    let address = addresses
        .into_iter()
        .find(|addr| {
            let ip = match addr.ip() {
                std::net::IpAddr::V6(v6) => v6
                    .to_ipv4_mapped()
                    .map(std::net::IpAddr::V4)
                    .unwrap_or(addr.ip()),
                ip => ip,
            };
            control
                .settings
                .security
                .upstream_cidr_allowlist
                .iter()
                .any(|net| net.contains(&ip))
        })
        .ok_or_else(|| {
            Error::explain(
                ErrorType::HTTPStatus(502),
                "Upstream IP denied by CIDR policy",
            )
        })?;
    let mut peer = HttpPeer::new(
        address,
        upstream.scheme == "https",
        upstream.hostname.trim_matches(['[', ']']).into(),
    );
    peer.options.connection_timeout = Some(Duration::from_millis(upstream.connect_timeout_ms));
    peer.options.total_connection_timeout =
        Some(Duration::from_millis(upstream.connect_timeout_ms));
    peer.options.read_timeout = Some(Duration::from_millis(upstream.read_timeout_ms));
    peer.options.write_timeout = Some(Duration::from_millis(upstream.read_timeout_ms));
    peer.options.verify_cert = true;
    peer.options.verify_hostname = true;
    Ok(Box::new(peer))
}
