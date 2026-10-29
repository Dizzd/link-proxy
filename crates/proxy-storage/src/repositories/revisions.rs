use anyhow::{Result, ensure};
use proxy_domain::*;
use sqlx::Transaction;

use crate::Store;
use crate::checksum;
impl Store {
    pub async fn state(&self) -> Result<(i64, i64)> {
        Ok(
            sqlx::query_as("SELECT active_revision,draft_generation FROM control_state WHERE id=1")
                .fetch_one(&self.pool)
                .await?,
        )
    }
    pub async fn revision(&self, version: i64) -> Result<Configuration> {
        let (data, hash): (String, String) =
            sqlx::query_as("SELECT data,checksum FROM config_revisions WHERE version=$1")
                .bind(version)
                .fetch_one(&self.pool)
                .await?;
        ensure!(
            checksum(data.as_bytes()) == hash,
            "database revision checksum mismatch"
        );
        Ok(serde_json::from_str(&data)?)
    }
    pub async fn committed(&self) -> Result<(i64, Configuration)> {
        let (version, _) = self.state().await?;
        if version == 0 {
            return Ok((
                0,
                Configuration {
                    schema_version: 1,
                    ..Default::default()
                },
            ));
        }
        Ok((version, self.revision(version).await?))
    }
    pub async fn commit_revision(
        tx: &mut Transaction<'_, sqlx::Any>,
        config: &Configuration,
        expected: i64,
        operation: &str,
    ) -> Result<i64> {
        let current: i64 =
            sqlx::query_scalar("SELECT active_revision FROM control_state WHERE id=1")
                .fetch_one(&mut **tx)
                .await?;
        ensure!(
            current == expected,
            "revision conflict: expected {expected}, current {current}"
        );
        let data = serde_json::to_string(config)?;
        let version:i64=sqlx::query_scalar("INSERT INTO config_revisions(status,checksum,data,created_by) VALUES('committed',$1,$2,$3) RETURNING version").bind(checksum(data.as_bytes())).bind(data).bind(crate::audit::current_actor()).fetch_one(&mut **tx).await?;
        sqlx::query("UPDATE control_state SET active_revision=$1 WHERE id=1")
            .bind(version)
            .execute(&mut **tx)
            .await?;
        sqlx::query("INSERT INTO audit_events(operation,revision,actor) VALUES($1,$2,$3)")
            .bind(operation)
            .bind(version)
            .bind(crate::audit::current_actor())
            .execute(&mut **tx)
            .await?;
        Ok(version)
    }
    pub async fn acknowledge(&self, version: i64) -> Result<()> {
        let mut tx = self.begin_write().await?;
        if Self::state_tx(&mut tx).await?.0 != version {
            return Ok(());
        }
        sqlx::query(
            "UPDATE config_revisions SET status='superseded' WHERE version<>$1 AND status='active'",
        )
        .bind(version)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE config_revisions SET status='active' WHERE version=$1")
            .bind(version)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn revisions(&self) -> Result<Vec<ConfigRevision>> {
        let rows: Vec<(i64,String,String,String)> = sqlx::query_as("SELECT version,status,checksum,created_at FROM config_revisions ORDER BY version DESC LIMIT 200").fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|(version, status, checksum, created_at)| ConfigRevision {
                version,
                status,
                checksum,
                created_at,
            })
            .collect())
    }
}
