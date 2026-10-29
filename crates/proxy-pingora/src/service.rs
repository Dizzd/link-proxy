pub(crate) use crate::context::ProxyContext;
pub(crate) use crate::response::{bad_request, proxy_error};
pub(crate) use async_trait::async_trait;
pub(crate) use bytes::Bytes;
pub(crate) use pingora::{
    Error, ErrorType, Result,
    http::{RequestHeader, ResponseHeader},
    prelude::{HttpPeer, ProxyHttp, Session},
};
pub(crate) use proxy_application::ControlPlane;
pub(crate) use proxy_domain::Action;
pub(crate) use proxy_routing::{
    RoutingSnapshot,
    conditions::{Clock, RequestContext, SystemClock},
    normalize_host, redirect_location, rewrite_path, validate_path,
};
pub(crate) use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct MappingProxy {
    pub control: Arc<ControlPlane>,
}
#[async_trait]
impl ProxyHttp for MappingProxy {
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        crate::upstream::select(&self.control, ctx).await
    }

    type CTX = ProxyContext;
    fn new_ctx(&self) -> Self::CTX {
        ProxyContext {
            snapshot: None,
            action: None,
            parameters: HashMap::new(),
            host: String::new(),
            bytes_read: 0,
            started: Instant::now(),
        }
    }
    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        self.control.metrics.requests.inc();
        let request = session.req_header();
        let host = request
            .headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .or_else(|| request.uri.host())
            .ok_or_else(|| bad_request("Host required"))?
            .to_string();
        normalize_host(&host).map_err(|_| bad_request("Invalid Host"))?;
        let path = request.uri.path();
        validate_path(path).map_err(|_| bad_request("Invalid path"))?;
        if path.len() > 4096 || request.uri.query().is_some_and(|q| q.len() > 8192) {
            session.respond_error(414).await?;
            return Ok(true);
        }
        if request
            .headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .is_some_and(|n| n > self.control.settings.security.max_proxy_body_bytes)
        {
            session.respond_error(413).await?;
            return Ok(true);
        }
        let snapshot = self.control.active.load_full();
        let timer = self.control.metrics.lookup_duration.start_timer();
        let route = snapshot.lookup(&host, path, request.method.as_str());
        timer.observe_duration();
        let Some(route) = route else {
            let status = if snapshot.path_exists(&host, path) {
                405
            } else {
                404
            };
            session.respond_error(status).await?;
            return Ok(true);
        };
        let context = RequestContext {
            now: SystemClock.now(),
            user_agent: request
                .headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default(),
            query: request.uri.query().unwrap_or_default(),
        };
        let action = route.mapping.action(&context).clone();
        ctx.parameters = route.parameters;
        if matches!(action, Action::Redirect { .. }) {
            let path = path.to_owned();
            let query = request.uri.query().map(str::to_owned);
            crate::redirect::respond(session, &action, &host, &path, query.as_deref()).await?;
            ctx.snapshot = Some(snapshot);
            return Ok(true);
        }
        ctx.host = host;
        ctx.action = Some(action);
        ctx.snapshot = Some(snapshot);
        Ok(false)
    }
    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        crate::filters::prepare(session, request, ctx)
    }

    async fn request_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<Bytes>,
        _end: bool,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        ctx.bytes_read = ctx
            .bytes_read
            .saturating_add(body.as_ref().map_or(0, Bytes::len));
        if ctx.bytes_read > self.control.settings.security.max_proxy_body_bytes {
            return Err(Error::explain(
                ErrorType::HTTPStatus(413),
                "Request body exceeds limit",
            ));
        }
        Ok(())
    }
    async fn logging(&self, session: &mut Session, error: Option<&Error>, ctx: &mut Self::CTX) {
        let elapsed = ctx.started.elapsed().as_secs_f64();
        self.control.metrics.request_duration.observe(elapsed);
        proxy_observability::tracing::request_complete(
            ctx.snapshot.as_ref().map(|s| s.revision),
            session.response_written().map(|r| r.status.as_u16()),
            elapsed,
            error.is_some(),
        );
    }
}
