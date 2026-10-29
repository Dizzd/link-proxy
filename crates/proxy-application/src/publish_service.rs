use anyhow::{Result, ensure};
use arc_swap::ArcSwap;
use proxy_cache::{CacheService, config::Settings};
use proxy_domain::*;
use proxy_observability::Metrics;
use proxy_routing::RoutingSnapshot;
use proxy_storage::Store;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

pub struct ControlPlane {
    pub store: Store,
    pub active: ArcSwap<RoutingSnapshot>,
    pub mutation: Mutex<()>,
    pub settings: Settings,
    pub metrics: Arc<Metrics>,
    pub cache: Arc<CacheService>,
    pub import_workers: Arc<Semaphore>,
    pub upload_slots: Arc<Semaphore>,
}
impl ControlPlane {
    pub async fn open(settings: Settings) -> Result<Arc<Self>> {
        let store = Store::connect(&settings.database_url).await?;
        let (revision, config) = store.committed().await?;
        let metrics = Arc::new(Metrics::new()?);
        let snapshot =
            RoutingSnapshot::compile(revision, config, &settings.security.redirect_host_allowlist)?;
        metrics.active_mappings.set(snapshot.mappings.len() as i64);
        metrics.revision.set(revision);
        let cache = Arc::new(CacheService::new(&settings.cache, metrics.clone())?);
        let workers = settings.imports.workers;
        let control = Arc::new(Self {
            store,
            active: ArcSwap::from_pointee(snapshot),
            mutation: Mutex::new(()),
            settings,
            metrics,
            cache,
            import_workers: Arc::new(Semaphore::new(workers)),
            upload_slots: Arc::new(Semaphore::new(workers)),
        });
        control.store.acknowledge(revision).await?;
        control.cache.status.write().await.active_revision = revision;
        Ok(control)
    }
    pub async fn compile(&self, revision: i64, config: Configuration) -> Result<RoutingSnapshot> {
        let metrics = self.metrics.clone();
        let allowlist = self.settings.security.redirect_host_allowlist.clone();
        tokio::task::spawn_blocking(move || {
            let _timer = metrics.compile_duration.start_timer();
            RoutingSnapshot::compile(revision, config, &allowlist)
        })
        .await?
    }
    pub async fn activate(&self, mut snapshot: RoutingSnapshot, revision: i64) {
        snapshot.revision = revision;
        self.metrics
            .active_mappings
            .set(snapshot.mappings.len() as i64);
        self.metrics.revision.set(revision);
        let configuration = snapshot.configuration.clone();
        self.active.store(Arc::new(snapshot));
        let cache = self.cache.clone();
        tokio::spawn(async move {
            cache.synchronize(revision, &configuration).await;
        });
        self.cache.status.write().await.active_revision = revision;
        if let Err(error) = self.store.acknowledge(revision).await {
            tracing::error!(%error, revision, "activation acknowledgement failed; startup will reconcile");
        }
    }
    pub async fn update_draft<F>(&self, edit: F) -> Result<Configuration>
    where
        F: FnOnce(&mut Configuration) -> Result<()>,
    {
        self.update_draft_for("draft_update", None, edit).await
    }
    pub async fn update_draft_for<F>(
        &self,
        operation: &str,
        resource_id: Option<&str>,
        edit: F,
    ) -> Result<Configuration>
    where
        F: FnOnce(&mut Configuration) -> Result<()>,
    {
        let _guard = self.mutation.lock().await;
        let base_state = self.store.state().await?;
        let mut config = self.store.draft().await?;
        edit(&mut config)?;
        self.compile(0, config.clone()).await?;
        let mut tx = self.store.begin_write().await?;
        Store::check_candidate(&mut tx, base_state).await?;
        Store::write_draft(&mut tx, &config).await?;
        sqlx::query("INSERT INTO audit_events(operation,revision,actor,resource_id) SELECT $1,active_revision,$2,$3 FROM control_state WHERE id=1").bind(operation).bind(proxy_storage::audit::current_actor()).bind(resource_id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(config)
    }
    pub async fn publish(&self, expected: i64, restore: Option<i64>) -> Result<i64> {
        let _guard = self.mutation.lock().await;
        let base_state = self.store.state().await?;
        let current = base_state.0;
        ensure!(current == expected, "revision conflict");
        let config = match restore {
            Some(version) => self.store.revision(version).await?,
            None => self.store.draft().await?,
        };
        let snapshot = self.compile(0, config.clone()).await?;
        let mut tx = self.store.begin_write().await?;
        Store::check_candidate(&mut tx, base_state).await?;
        if restore.is_some() {
            Store::write_draft(&mut tx, &config).await?;
        }
        let revision = Store::commit_revision(
            &mut tx,
            &config,
            expected,
            if restore.is_some() {
                "restore"
            } else {
                "publish"
            },
        )
        .await?;
        tx.commit().await?;
        self.activate(snapshot, revision).await;
        Ok(revision)
    }
    pub async fn reconcile(&self) -> Result<()> {
        let _guard = self.mutation.lock().await;
        let (revision, _) = self.store.state().await?;
        let local = self.active.load().revision;
        self.metrics.revision_lag.set((revision - local).max(0));
        if revision != local {
            let config = self.store.revision(revision).await?;
            let snapshot = self.compile(revision, config).await?;
            self.activate(snapshot, revision).await;
        }
        // Snapshot Arc is retained briefly, no ArcSwap guard held across Redis I/O.
        let snapshot = self.active.load_full();
        drop(_guard);
        self.cache
            .synchronize(snapshot.revision, &snapshot.configuration)
            .await;
        self.store
            .cleanup(self.settings.imports.retention_hours)
            .await?;
        Ok(())
    }
}
