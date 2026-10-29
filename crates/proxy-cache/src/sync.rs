use super::*;
#[derive(Clone, Debug, Serialize)]
pub struct CacheStatus {
    pub mode: CacheMode,
    pub connected: bool,
    pub active_revision: i64,
    pub database_revision: i64,
    pub last_sync: Option<String>,
    pub error: Option<String>,
}
pub struct CacheService {
    backend: Arc<dyn ConfigurationCache>,
    pub status: RwLock<CacheStatus>,
    metrics: Arc<Metrics>,
}
impl CacheService {
    pub fn new(settings: &CacheSettings, metrics: Arc<Metrics>) -> Result<Self> {
        let backend: Arc<dyn ConfigurationCache> = match settings.mode {
            CacheMode::Memory => Arc::new(MemoryConfigurationCache::default()),
            CacheMode::MemoryRedis => {
                ensure!(
                    settings.redis.enabled,
                    "memory_redis requires cache.redis.enabled=true"
                );
                #[cfg(feature = "redis-cache")]
                {
                    Arc::new(RedisConfigurationCache {
                        settings: settings.redis.clone(),
                    })
                }
                #[cfg(not(feature = "redis-cache"))]
                {
                    anyhow::bail!("build with --features redis-cache to enable Redis")
                }
            }
        };
        Ok(Self {
            backend,
            status: RwLock::new(CacheStatus {
                mode: settings.mode.clone(),
                connected: settings.mode == CacheMode::Memory,
                active_revision: 0,
                database_revision: 0,
                last_sync: None,
                error: None,
            }),
            metrics,
        })
    }
    pub async fn synchronize(&self, revision: i64, config: &Configuration) {
        let _timer = self.metrics.sync_duration.start_timer();
        let result = async {
            let envelope = CachedConfiguration::new(revision, config)?;
            if let Some(cached) = self.backend.load(revision).await? {
                cached.validate(revision, &envelope.checksum)?;
            }
            self.backend.save(envelope).await
        }
        .await;
        let mut status = self.status.write().await;
        status.database_revision = revision;
        status.last_sync = Some(chrono::Utc::now().to_rfc3339());
        match result {
            Ok(()) => {
                status.connected = true;
                status.error = None;
                self.metrics
                    .redis_connected
                    .set(i64::from(status.mode == CacheMode::MemoryRedis));
            }
            Err(error) => {
                status.connected = false;
                status.error = Some(
                    "Configuration cache unavailable or invalid; database remains authoritative"
                        .into(),
                );
                self.metrics.redis_connected.set(0);
                self.metrics.redis_errors.inc();
                tracing::warn!(error_kind = %error.root_cause().to_string().split(':').next().unwrap_or("cache"), "configuration cache degraded");
            }
        }
    }
}

impl CacheService {
    /// Subscribe to advisory revision hints. Every hint is checked against SQLite.
    pub fn notifications(
        settings: &config::CacheSettings,
    ) -> Option<tokio::sync::mpsc::Receiver<i64>> {
        if settings.mode != config::CacheMode::MemoryRedis {
            return None;
        }
        #[cfg(feature = "redis-cache")]
        {
            use futures_util::StreamExt;
            let settings = settings.redis.clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                loop {
                    let listen = async {
                        let url = std::env::var(&settings.url_env)?;
                        let client = ::redis::Client::open(url)?;
                        let mut connection = tokio::time::timeout(
                            std::time::Duration::from_millis(settings.connect_timeout_ms),
                            client.get_async_pubsub(),
                        )
                        .await??;
                        tokio::time::timeout(
                            std::time::Duration::from_millis(settings.command_timeout_ms),
                            connection.subscribe(format!("{}config:updates", settings.key_prefix)),
                        )
                        .await??;
                        let mut messages = connection.on_message();
                        while let Some(message) = messages.next().await {
                            if let Ok(revision) = message.get_payload::<i64>() {
                                let _ = sender.try_send(revision);
                            }
                        }
                        Ok::<(), anyhow::Error>(())
                    };
                    tokio::select! { _=sender.closed()=>return, result=listen=> { if result.is_err() {tracing::warn!("Redis notification listener unavailable; periodic SQLite reconciliation continues");} } }
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            });
            Some(receiver)
        }
        #[cfg(not(feature = "redis-cache"))]
        {
            None
        }
    }
}
