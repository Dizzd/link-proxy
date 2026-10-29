pub mod config;
use crate::config::{CacheMode, CacheSettings};
use anyhow::{Result, ensure};
use proxy_domain::Configuration;
use proxy_observability::Metrics;
use proxy_storage::checksum;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Serialize, Deserialize)]
pub struct CachedConfiguration {
    pub schema_version: u32,
    pub revision: i64,
    pub checksum: String,
    pub data: String,
}
impl CachedConfiguration {
    pub fn new(revision: i64, config: &Configuration) -> Result<Self> {
        let data = serde_json::to_string(config)?;
        Ok(Self {
            schema_version: 1,
            revision,
            checksum: checksum(data.as_bytes()),
            data,
        })
    }
    pub fn validate(&self, revision: i64, expected_checksum: &str) -> Result<Configuration> {
        if self.schema_version != 1
            || self.revision != revision
            || self.checksum != expected_checksum
            || checksum(self.data.as_bytes()) != self.checksum
        {
            return Err(error::CacheIntegrityError.into());
        }
        Ok(serde_json::from_str(&self.data)?)
    }
}
type CacheFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
pub trait ConfigurationCache: Send + Sync {
    fn load(&self, revision: i64) -> CacheFuture<'_, Option<CachedConfiguration>>;
    fn save(&self, config: CachedConfiguration) -> CacheFuture<'_, ()>;
}
pub mod memory;
pub use memory::*;
#[cfg(feature = "redis-cache")]
pub mod redis;
#[cfg(feature = "redis-cache")]
pub use redis::*;
pub mod sync;
pub use sync::*;
pub mod error;
