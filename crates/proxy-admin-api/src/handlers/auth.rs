use crate::auth::{AuthError, COOKIE_NAME, csrf_token, same_origin, session_cookie, token_hash};
use crate::router::*;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
#[derive(Deserialize)]
pub(crate) struct Login {
    username: String,
    password: String,
}
#[derive(Deserialize)]
pub(crate) struct ChangePassword {
    current_password: String,
    new_password: String,
}
fn username(input: &str) -> Result<String> {
    let value = input.trim().to_ascii_lowercase();
    ensure!(
        (3..=64).contains(&value.len())
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-@".contains(&b)),
        "username must contain 3..64 ASCII letters, digits, underscore, dot, hyphen or @"
    );
    Ok(value)
}
fn password(input: &str) -> Result<()> {
    ensure!(
        (12..=1024).contains(&input.len()) && !input.trim().is_empty(),
        "password must contain 12..1024 bytes and cannot be blank"
    );
    Ok(())
}
fn hash_password(value: &str) -> Result<String> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    Argon2::default()
        .hash_password(value.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| anyhow::anyhow!("Password hashing failed"))
}
fn verify(value: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(value.as_bytes(), &hash)
            .is_ok()
    })
}
fn cookie(token: &str, ttl: i64, secure: bool) -> String {
    format!(
        "{COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={ttl}{}",
        if secure { "; Secure" } else { "" }
    )
}
async fn required_session(
    state: &ApiState,
    headers: &HeaderMap,
) -> ApiResult<(String, proxy_storage::repositories::auth::AdminSession)> {
    let token = session_cookie(headers).ok_or(AuthError::Required)?;
    let session = state
        .control
        .store
        .session(&token_hash(&token))
        .await?
        .ok_or(AuthError::Required)?;
    Ok((token, session))
}
pub(crate) async fn login(
    State(s): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<Login>,
) -> ApiResult<Response> {
    ensure!(
        input.username.len() <= 64 && input.password.len() <= 1024,
        "credentials exceed size limits"
    );
    if !same_origin(&headers) {
        return Err(AuthError::Csrf.into());
    }
    let mut rate = s.login_rate.lock().await;
    if rate.0.elapsed().as_secs() >= 60 {
        *rate = (Instant::now(), 0);
    }
    if rate.1 >= s.control.settings.auth.login_requests_per_minute {
        return Ok((
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "60")],
            Json(
                json!({"error":{"code":"429","message":"Too many sign-in attempts; retry later"}}),
            ),
        )
            .into_response());
    }
    rate.1 += 1;
    drop(rate);
    let permit = s
        .auth_workers
        .clone()
        .try_acquire_owned()
        .map_err(|_| anyhow::anyhow!("authentication worker capacity reached"))?;
    let name = username(&input.username).map_err(|_| AuthError::Credentials)?;
    let user = s.control.store.admin_user(&name).await?;
    let hash = user.as_ref().map(|u| u.password_hash.clone());
    let valid = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        match hash {
            Some(hash) => verify(&input.password, &hash),
            None => {
                let _ = hash_password(&input.password);
                false
            }
        }
    })
    .await?;
    let Some(user) = user.filter(|user| user.enabled && valid) else {
        s.control.store.audit_auth_failure(&name).await?;
        return Err(AuthError::Credentials.into());
    };
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let expires_at = chrono::Utc::now().timestamp() + s.control.settings.auth.session_ttl_seconds;
    if !s
        .control
        .store
        .create_session(&user, &token_hash(&token), expires_at)
        .await?
    {
        return Err(AuthError::Credentials.into());
    }
    Ok(([(header::SET_COOKIE,cookie(&token,s.control.settings.auth.session_ttl_seconds,s.control.settings.auth.secure_cookies)),(header::CACHE_CONTROL,"no-store".into())],Json(json!({"user":{"id":user.id,"username":user.username},"csrf_token":csrf_token(&token),"expires_at":expires_at}))).into_response())
}
pub(crate) async fn session(State(s): State<ApiState>, headers: HeaderMap) -> ApiResult<Response> {
    let (token, session) = required_session(&s, &headers).await?;
    Ok(([(header::CACHE_CONTROL,"no-store")],Json(json!({"user":{"id":session.user_id,"username":session.username},"csrf_token":csrf_token(&token),"expires_at":session.expires_at}))).into_response())
}
pub(crate) async fn logout(State(s): State<ApiState>, headers: HeaderMap) -> ApiResult<Response> {
    let (token, session) = required_session(&s, &headers).await?;
    s.control
        .store
        .revoke_session(&token_hash(&token), &session)
        .await?;
    Ok((
        StatusCode::NO_CONTENT,
        [
            (
                header::SET_COOKIE,
                cookie("", 0, s.control.settings.auth.secure_cookies),
            ),
            (header::CACHE_CONTROL, "no-store".into()),
        ],
    )
        .into_response())
}
pub(crate) async fn change_password(
    State(s): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<ChangePassword>,
) -> ApiResult<Response> {
    password(&input.new_password)?;
    ensure!(
        input.current_password.len() <= 1024,
        "credentials exceed size limits"
    );
    let (_, session) = required_session(&s, &headers).await?;
    let user = s
        .control
        .store
        .admin_user(&session.username)
        .await?
        .ok_or(AuthError::Required)?;
    let permit = s
        .auth_workers
        .clone()
        .try_acquire_owned()
        .map_err(|_| anyhow::anyhow!("authentication worker capacity reached"))?;
    let old_hash = user.password_hash.clone();
    let hash = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        if !verify(&input.current_password, &old_hash) {
            return Ok::<_, anyhow::Error>(None);
        }
        Ok(Some(hash_password(&input.new_password)?))
    })
    .await??;
    let Some(hash) = hash else {
        return Err(AuthError::Password.into());
    };
    s.control.store.change_admin_password(&user, &hash).await?;
    Ok((
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            cookie("", 0, s.control.settings.auth.secure_cookies),
        )],
    )
        .into_response())
}
/// Bootstrap only an empty user table. Existing credentials are never reset at startup.
pub async fn bootstrap_admin(control: &ControlPlane, reset: bool) -> Result<()> {
    let count = control.store.admin_count().await?;
    if count > 0 && !reset {
        return Ok(());
    }
    let name = username(
        &std::env::var(&control.settings.auth.username_env).map_err(|_| {
            anyhow::anyhow!(
                "Set ADMIN_USERNAME and ADMIN_PASSWORD to initialize the first administrator"
            )
        })?,
    )?;
    let secret = std::env::var(&control.settings.auth.password_env)
        .map_err(|_| anyhow::anyhow!("Administrator password environment variable is missing"))?;
    password(&secret)?;
    let hash = tokio::task::spawn_blocking(move || hash_password(&secret)).await??;
    if reset {
        let user = control
            .store
            .admin_user(&name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("administrator not found"))?;
        control.store.change_admin_password(&user, &hash).await?;
        tracing::info!("Administrator password reset; existing sessions revoked");
    } else {
        control
            .store
            .bootstrap_admin(&uuid::Uuid::new_v4().to_string(), &name, &hash)
            .await?;
        tracing::info!("Initial administrator created");
    }
    Ok(())
}
