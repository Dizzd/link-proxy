use super::*;
#[derive(Default)]
pub struct MemoryConfigurationCache {
    value: RwLock<Option<CachedConfiguration>>,
}
impl ConfigurationCache for MemoryConfigurationCache {
    fn load(&self, revision: i64) -> CacheFuture<'_, Option<CachedConfiguration>> {
        Box::pin(async move {
            Ok(self
                .value
                .read()
                .await
                .clone()
                .filter(|v| v.revision == revision))
        })
    }
    fn save(&self, config: CachedConfiguration) -> CacheFuture<'_, ()> {
        Box::pin(async move {
            *self.value.write().await = Some(config);
            Ok(())
        })
    }
}
