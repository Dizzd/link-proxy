use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use sqlx::{Any, Transaction};

use crate::Store;

pub const REVISION_PRUNE_BATCH_SIZE: i64 = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionPrunePreview {
    pub keep_latest: u32,
    pub database_revision: i64,
    pub active_revision: i64,
    pub total_revisions: i64,
    pub eligible_count: i64,
    pub versions: Vec<i64>,
}

impl Store {
    async fn revision_prune_preview_tx(
        tx: &mut Transaction<'_, Any>,
        keep_latest: u32,
        active_revision: i64,
    ) -> Result<RevisionPrunePreview> {
        ensure!(
            (1..=10_000).contains(&keep_latest),
            "keep_latest must be between 1 and 10000"
        );
        let (database_revision, _) = Self::state_tx(tx).await?;
        let total_revisions = sqlx::query_scalar("SELECT COUNT(*) FROM config_revisions")
            .fetch_one(&mut **tx)
            .await?;
        let cutoff: Option<i64> = sqlx::query_scalar(
            "SELECT version FROM config_revisions ORDER BY version DESC LIMIT 1 OFFSET $1",
        )
        .bind(i64::from(keep_latest) - 1)
        .fetch_optional(&mut **tx)
        .await?;
        let cutoff = cutoff.unwrap_or(0);
        let eligible_count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM config_revisions WHERE status='superseded' AND version<>$1 AND version<>$2 AND version<$3",
        ).bind(database_revision).bind(active_revision).bind(cutoff)
            .fetch_one(&mut **tx).await?;
        let versions = sqlx::query_scalar(
            "SELECT version FROM config_revisions WHERE status='superseded' AND version<>$1 AND version<>$2 AND version<$3 ORDER BY version LIMIT $4",
        ).bind(database_revision).bind(active_revision).bind(cutoff).bind(REVISION_PRUNE_BATCH_SIZE)
            .fetch_all(&mut **tx).await?;
        Ok(RevisionPrunePreview {
            keep_latest,
            database_revision,
            active_revision,
            total_revisions,
            eligible_count,
            versions,
        })
    }

    pub async fn preview_revision_prune(
        &self,
        keep_latest: u32,
        active_revision: i64,
    ) -> Result<RevisionPrunePreview> {
        let mut tx = self.begin_write().await?;
        let preview =
            Self::revision_prune_preview_tx(&mut tx, keep_latest, active_revision).await?;
        tx.commit().await?;
        Ok(preview)
    }

    pub async fn prune_revisions(
        &self,
        expected: &RevisionPrunePreview,
        active_revision: i64,
    ) -> Result<u64> {
        let mut tx = self.begin_write().await?;
        let preview =
            Self::revision_prune_preview_tx(&mut tx, expected.keep_latest, active_revision).await?;
        ensure!(
            preview == *expected,
            "revision conflict: cleanup preview changed; preview again before deleting"
        );
        let Some(last) = preview.versions.last() else {
            tx.commit().await?;
            return Ok(0);
        };
        // Candidates are the oldest eligible rows. The bounded range excludes both pointers,
        // and committed/active statuses stay protected even if acknowledgement was interrupted.
        let deleted = sqlx::query("DELETE FROM config_revisions WHERE status='superseded' AND version<>$1 AND version<>$2 AND version<=$3")
            .bind(preview.database_revision).bind(active_revision).bind(last)
            .execute(&mut *tx).await?.rows_affected();
        ensure!(
            deleted == preview.versions.len() as u64,
            "revision conflict: cleanup candidates changed"
        );
        let resource = serde_json::json!({
            "keep_latest": preview.keep_latest, "deleted_count": deleted,
            "first_version": preview.versions.first(), "last_version": last,
        })
        .to_string();
        sqlx::query("INSERT INTO audit_events(operation,revision,actor,resource_id) VALUES('revision_prune',$1,$2,$3)")
            .bind(preview.database_revision).bind(crate::audit::current_actor()).bind(resource)
            .execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(deleted)
    }
}
