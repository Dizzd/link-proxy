use super::*;
#[cfg(feature = "redis-cache")]
pub struct RedisConfigurationCache {
    pub(crate) settings: crate::config::RedisSettings,
}
#[cfg(feature = "redis-cache")]
impl RedisConfigurationCache {
    async fn connection(&self) -> Result<::redis::aio::MultiplexedConnection> {
        let url = std::env::var(&self.settings.url_env)
            .map_err(|_| anyhow::anyhow!("Redis URL environment variable is missing"))?;
        let client =
            ::redis::Client::open(url).map_err(|_| anyhow::anyhow!("invalid Redis URL"))?;
        Ok(tokio::time::timeout(
            std::time::Duration::from_millis(self.settings.connect_timeout_ms),
            client.get_multiplexed_async_connection(),
        )
        .await??)
    }
}
#[cfg(feature = "redis-cache")]
impl ConfigurationCache for RedisConfigurationCache {
    fn load(&self, revision: i64) -> CacheFuture<'_, Option<CachedConfiguration>> {
        Box::pin(async move {
            let mut connection = self.connection().await?;
            let value: Option<String> = tokio::time::timeout(
                std::time::Duration::from_millis(self.settings.command_timeout_ms),
                ::redis::cmd("GET")
                    .arg(format!(
                        "{}config:revision:{revision}",
                        self.settings.key_prefix
                    ))
                    .query_async(&mut connection),
            )
            .await??;
            value
                .map(|s| serde_json::from_str(&s).map_err(Into::into))
                .transpose()
        })
    }
    fn save(&self, config: CachedConfiguration) -> CacheFuture<'_, ()> {
        Box::pin(async move {
            let mut connection = self.connection().await?;
            // CAS protects latest metadata from delayed older writers. Version keys expire independently.
            let script = ::redis::Script::new(
                "local current = tonumber(redis.call('GET', KEYS[2]) or '-1'); redis.call('SET', KEYS[1], ARGV[2], 'EX', 604800); if tonumber(ARGV[1]) > current then redis.call('SET', KEYS[2], ARGV[1]); redis.call('PUBLISH', KEYS[3], ARGV[1]); end; return 1",
            );
            let _: i64 = tokio::time::timeout(
                std::time::Duration::from_millis(self.settings.command_timeout_ms),
                script
                    .key(format!(
                        "{}config:revision:{}",
                        self.settings.key_prefix, config.revision
                    ))
                    .key(format!("{}config:latest", self.settings.key_prefix))
                    .key(format!("{}config:updates", self.settings.key_prefix))
                    .arg(config.revision)
                    .arg(serde_json::to_string(&config)?)
                    .invoke_async(&mut connection),
            )
            .await??;
            Ok(())
        })
    }
}
