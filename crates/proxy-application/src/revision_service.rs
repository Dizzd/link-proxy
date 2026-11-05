use crate::ControlPlane;
pub use proxy_storage::repositories::revision_cleanup::RevisionPrunePreview;
impl ControlPlane {
    pub async fn preview_revision_prune(
        &self,
        keep_latest: u32,
    ) -> anyhow::Result<RevisionPrunePreview> {
        let _guard = self.mutation.lock().await;
        let active_revision = self.active.load().revision;
        self.store
            .preview_revision_prune(keep_latest, active_revision)
            .await
    }

    pub async fn prune_revisions(&self, preview: &RevisionPrunePreview) -> anyhow::Result<u64> {
        let _guard = self.mutation.lock().await;
        let active_revision = self.active.load().revision;
        self.store.prune_revisions(preview, active_revision).await
    }

    pub async fn restore_revision(
        &self,
        version: i64,
        expected_revision: i64,
    ) -> anyhow::Result<i64> {
        self.publish(expected_revision, Some(version)).await
    }
}
