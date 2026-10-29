use crate::router::*;
pub(crate) async fn health(State(s): State<ApiState>) -> Json<Value> {
    Json(json!({"status":"ok","revision":s.control.active.load().revision}))
}
pub(crate) async fn metrics(State(s): State<ApiState>) -> ApiResult<Response> {
    Ok((
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        s.control.metrics.render()?,
    )
        .into_response())
}

pub(crate) async fn ready(State(s): State<ApiState>) -> Response {
    let result = async {
        let mut address: std::net::SocketAddr = s.control.settings.proxy_listen.parse()?;
        if address.ip().is_unspecified() {
            address.set_ip(if address.is_ipv4() {
                std::net::Ipv4Addr::LOCALHOST.into()
            } else {
                std::net::Ipv6Addr::LOCALHOST.into()
            });
        }
        tokio::time::timeout(
            std::time::Duration::from_millis(200),
            tokio::net::TcpStream::connect(address),
        )
        .await??;
        Ok::<(), anyhow::Error>(())
    }
    .await;
    let status = if result.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status,Json(json!({"status":if result.is_ok(){"ready"}else{"proxy_unavailable"},"revision":s.control.active.load().revision}))).into_response()
}
