use crate::router::*;
use axum::http::Method;
pub(crate) const COOKIE_NAME: &str = "dup_admin_session";
#[derive(Debug, thiserror::Error)]
pub(crate) enum AuthError {
    #[error("Invalid username or password")]
    Credentials,
    #[error("Authentication required")]
    Required,
    #[error("CSRF validation failed")]
    Csrf,
    #[error("Current password is incorrect")]
    Password,
}
impl AuthError {
    pub(crate) fn status(&self) -> StatusCode {
        match self {
            Self::Credentials | Self::Required => StatusCode::UNAUTHORIZED,
            Self::Csrf | Self::Password => StatusCode::FORBIDDEN,
        }
    }
}
pub(crate) fn token_hash(token: &str) -> String {
    proxy_storage::checksum(token.as_bytes())
}
pub(crate) fn csrf_token(token: &str) -> String {
    proxy_storage::checksum(format!("csrf:{token}").as_bytes())
}
pub(crate) fn session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    let mut matches = cookie
        .split(';')
        .filter_map(|item| item.trim().strip_prefix(&format!("{COOKIE_NAME}=")));
    let token = matches.next()?;
    if matches.next().is_some()
        || token.len() != 64
        || !token.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return None;
    }
    Some(token.to_owned())
}
pub(crate) fn same_origin(headers: &HeaderMap) -> bool {
    if headers
        .get("sec-fetch-site")
        .is_some_and(|v| v == "cross-site")
    {
        return false;
    }
    let Some(origin) = headers.get(header::ORIGIN) else {
        return true;
    };
    let Some(host) = headers.get(header::HOST).and_then(|h| h.to_str().ok()) else {
        return false;
    };
    let Ok(origin) = origin.to_str().unwrap_or_default().parse::<url::Url>() else {
        return false;
    };
    let Ok(authority) = host.parse::<http::uri::Authority>() else {
        return false;
    };
    matches!(origin.scheme(), "http" | "https")
        && origin
            .host_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(authority.host()))
        && origin.port_or_known_default()
            == Some(
                authority
                    .port_u16()
                    .unwrap_or(if origin.scheme() == "https" { 443 } else { 80 }),
            )
}
fn failure(status: StatusCode, message: &str) -> Response {
    (
        status,
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({"error":{"code":status.as_str(),"message":message}})),
    )
        .into_response()
}
pub(crate) async fn authorize(
    State(state): State<ApiState>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut rate = state.rate.lock().await;
    if rate.0.elapsed().as_secs() >= 60 {
        *rate = (Instant::now(), 0);
    }
    if rate.1 >= state.control.settings.security.admin_requests_per_minute {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "60")],
            Json(json!({"error":{"code":"429","message":"Admin request rate exceeded"}})),
        )
            .into_response();
    }
    rate.1 += 1;
    drop(rate);
    let supplied = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or_default();
    let supplied_hash: [u8; 32] = Sha256::digest(supplied.as_bytes()).into();
    let actor = if state
        .token_hash
        .is_some_and(|hash| bool::from(hash.ct_eq(&supplied_hash)))
    {
        "admin-token".to_owned()
    } else {
        let Some(token) = session_cookie(request.headers()) else {
            return failure(StatusCode::UNAUTHORIZED, "Authentication required");
        };
        let session = match state.control.store.session(&token_hash(&token)).await {
            Ok(Some(session)) => session,
            Ok(None) => return failure(StatusCode::UNAUTHORIZED, "Session expired or revoked"),
            Err(error) => return crate::error::ApiError::from(error).into_response(),
        };
        if !matches!(
            *request.method(),
            Method::GET | Method::HEAD | Method::OPTIONS
        ) {
            let supplied = request
                .headers()
                .get("x-csrf-token")
                .and_then(|h| h.to_str().ok())
                .unwrap_or_default();
            let expected = csrf_token(&token);
            let valid = bool::from(
                Sha256::digest(supplied.as_bytes()).ct_eq(&Sha256::digest(expected.as_bytes())),
            );
            if !valid || !same_origin(request.headers()) {
                return failure(StatusCode::FORBIDDEN, "CSRF validation failed");
            }
        }
        session.username
    };
    // Ordinary JSON management requests are smaller than bounded multipart uploads.
    if request.method() != Method::GET && request.uri().path() != "/api/v1/imports" {
        let (parts, body) = request.into_parts();
        let bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
            Ok(bytes) => bytes,
            Err(_) => return failure(StatusCode::PAYLOAD_TOO_LARGE, "Request body exceeds limit"),
        };
        request = Request::from_parts(parts, axum::body::Body::from(bytes));
    }
    proxy_storage::audit::scope(
        actor,
        async move { structured(next.run(request).await).await },
    )
    .await
}
pub(crate) async fn structured(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        http::HeaderValue::from_static("no-store"),
    );
    let status = response.status();
    if !(status.is_client_error() || status.is_server_error())
        || response
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|v| v.as_bytes().starts_with(b"application/json"))
    {
        return response;
    }
    let bytes = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap_or_default();
    let message = if status.is_server_error() {
        "Internal server error".to_owned()
    } else {
        let text = String::from_utf8_lossy(&bytes);
        if text.is_empty() {
            status
                .canonical_reason()
                .unwrap_or("Request failed")
                .to_owned()
        } else {
            text.into_owned()
        }
    };
    failure(status, &message)
}
