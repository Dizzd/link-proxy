use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub admin_listen: String,
    pub admin_static_dir: String,
    pub proxy_listen: String,
    pub database_url: String,
    pub database_url_env: String,
    pub auth: AuthSettings,
    pub admin_token_env: String,
    pub allow_remote_admin: bool,
    pub cache: CacheSettings,
    pub security: SecuritySettings,
    pub imports: ImportLimits,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            admin_listen: "127.0.0.1:9000".into(),
            admin_static_dir: "apps/admin-web/dist".into(),
            proxy_listen: "0.0.0.0:8080".into(),
            database_url: "sqlite://data/proxy.db".into(),
            database_url_env: "DATABASE_URL".into(),
            auth: AuthSettings::default(),
            admin_token_env: "ADMIN_API_TOKEN".into(),
            allow_remote_admin: false,
            cache: CacheSettings::default(),
            security: SecuritySettings::default(),
            imports: ImportLimits::default(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheMode {
    #[default]
    Memory,
    MemoryRedis,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CacheSettings {
    pub mode: CacheMode,
    pub memory: MemorySettings,
    pub redis: RedisSettings,
}
impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            mode: CacheMode::Memory,
            memory: MemorySettings { enabled: true },
            redis: RedisSettings::default(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemorySettings {
    pub enabled: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct RedisSettings {
    pub enabled: bool,
    pub url_env: String,
    pub key_prefix: String,
    pub connect_timeout_ms: u64,
    pub command_timeout_ms: u64,
    pub sync_interval_seconds: u64,
}
impl Default for RedisSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            url_env: "REDIS_URL".into(),
            key_prefix: "dynamic_proxy:".into(),
            connect_timeout_ms: 1000,
            command_timeout_ms: 1000,
            sync_interval_seconds: 30,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SecuritySettings {
    pub upstream_cidr_allowlist: Vec<ipnet::IpNet>,
    pub redirect_host_allowlist: Vec<String>,
    pub max_proxy_body_bytes: usize,
    pub admin_requests_per_minute: u32,
}
impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            upstream_cidr_allowlist: Vec::new(),
            redirect_host_allowlist: Vec::new(),
            max_proxy_body_bytes: 16 * 1024 * 1024,
            admin_requests_per_minute: 6000,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ImportLimits {
    pub max_upload_bytes: usize,
    pub max_records: usize,
    pub max_record_bytes: usize,
    pub max_uncompressed_bytes: u64,
    pub workers: usize,
    pub max_jobs: usize,
    pub retention_hours: i64,
}
impl Default for ImportLimits {
    fn default() -> Self {
        Self {
            max_upload_bytes: 64 * 1024 * 1024,
            max_records: 50_000,
            max_record_bytes: 64 * 1024,
            max_uncompressed_bytes: 256 * 1024 * 1024,
            workers: 2,
            max_jobs: 100,
            retention_hours: 72,
        }
    }
}
impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        let mut settings: Self = toml::from_str(&std::fs::read_to_string(path)?)?;
        if let Ok(url) = std::env::var(&settings.database_url_env) {
            ensure!(
                !url.is_empty(),
                "database URL environment variable is empty"
            );
            settings.database_url = url;
        }
        ensure!(
            (300..=604800).contains(&settings.auth.session_ttl_seconds)
                && settings.auth.login_requests_per_minute > 0,
            "invalid authentication limits"
        );
        let admin: SocketAddr = settings.admin_listen.parse()?;
        ensure!(
            admin.ip().is_loopback() || settings.allow_remote_admin,
            "remote administration requires allow_remote_admin=true and a protected network"
        );
        ensure!(
            settings.cache.memory.enabled,
            "active local memory snapshot must be enabled"
        );
        ensure!(
            settings.imports.workers > 0
                && settings.imports.max_records > 0
                && settings.imports.max_upload_bytes > 0
                && settings.imports.retention_hours > 0
                && settings.imports.max_jobs > 0,
            "invalid import limits"
        );
        ensure!(
            settings.cache.redis.sync_interval_seconds > 0,
            "sync interval must be positive"
        );
        Ok(settings)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AuthSettings {
    pub username_env: String,
    pub password_env: String,
    pub session_ttl_seconds: i64,
    pub secure_cookies: bool,
    pub api_token_enabled: bool,
    pub login_requests_per_minute: u32,
}
impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            username_env: "ADMIN_USERNAME".into(),
            password_env: "ADMIN_PASSWORD".into(),
            session_ttl_seconds: 43200,
            secure_cookies: false,
            api_token_enabled: false,
            login_requests_per_minute: 20,
        }
    }
}
