use proxy_domain::Configuration;
use std::{future::Future, pin::Pin};
pub type RepositoryFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>;
/// Persistence boundary implementable by SQLite or PostgreSQL adapters.
pub trait ConfigurationRepository: Send + Sync {
    fn committed_configuration(&self) -> RepositoryFuture<'_, (i64, Configuration)>;
}
impl ConfigurationRepository for crate::Store {
    fn committed_configuration(&self) -> RepositoryFuture<'_, (i64, Configuration)> {
        Box::pin(self.committed())
    }
}
