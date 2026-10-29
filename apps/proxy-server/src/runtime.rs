use proxy_application::ControlPlane;
use std::sync::Arc;
pub async fn reconciliation(control: Arc<ControlPlane>) {
    let mut hints = proxy_cache::CacheService::notifications(&control.settings.cache);
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(
        control.settings.cache.redis.sync_interval_seconds,
    ));
    loop {
        tokio::select! {
            _ = interval.tick() => {},
            _ = async { if let Some(receiver)=hints.as_mut() {receiver.recv().await;} else {std::future::pending::<()>().await;} } => {},
        }
        if let Err(error) = control.reconcile().await {
            tracing::error!(%error,"configuration reconciliation failed; retaining valid local snapshot");
        }
    }
}
