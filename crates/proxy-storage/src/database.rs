use crate::{DatabaseBackend, Store};
use anyhow::{Result, ensure};
use sqlx::any::AnyPoolOptions;
impl Store {
    pub async fn connect(url: &str) -> Result<Self> {
        sqlx::any::install_default_drivers();
        let backend = if url.starts_with("sqlite:") {
            DatabaseBackend::Sqlite
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            DatabaseBackend::Postgres
        } else {
            anyhow::bail!("database_url must use sqlite, postgres or postgresql scheme")
        };
        if let Some(path) = url
            .strip_prefix("sqlite://")
            .and_then(|s| s.split('?').next())
            && let Some(parent) = std::path::Path::new(path)
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let memory = url.contains(":memory:");
        let connection_url =
            if backend == DatabaseBackend::Sqlite && !memory && !url.contains("mode=") {
                format!("{url}{}mode=rwc", if url.contains('?') { "&" } else { "?" })
            } else {
                url.to_owned()
            };
        let pool = AnyPoolOptions::new()
            .max_connections(if memory { 1 } else { 10 })
            .acquire_timeout(std::time::Duration::from_secs(15))
            .after_connect(move |connection, _| {
                Box::pin(async move {
                    if backend == DatabaseBackend::Sqlite {
                        sqlx::query("PRAGMA foreign_keys=ON")
                            .execute(&mut *connection)
                            .await?;
                        sqlx::query("PRAGMA busy_timeout=10000")
                            .execute(&mut *connection)
                            .await?;
                        if !memory {
                            sqlx::query("PRAGMA journal_mode=WAL")
                                .execute(&mut *connection)
                                .await?;
                        }
                    } else {
                        sqlx::query("SET TIME ZONE 'UTC'")
                            .execute(&mut *connection)
                            .await?;
                    }
                    Ok(())
                })
            })
            .connect(&connection_url)
            .await?;
        match backend {
            DatabaseBackend::Sqlite => sqlx::migrate!("../../migrations/sqlite").run(&pool).await?,
            DatabaseBackend::Postgres => {
                sqlx::migrate!("../../migrations/postgres")
                    .run(&pool)
                    .await?
            }
        }
        sqlx::query("UPDATE import_jobs SET status='failed',result='Interrupted before activation' WHERE status IN ('pending','validating')").execute(&pool).await?;
        sqlx::query("UPDATE import_jobs SET status=CASE WHEN committed_revision IS NOT NULL THEN 'completed' ELSE 'failed' END WHERE status='committing'").execute(&pool).await?;
        Ok(Self { pool, backend })
    }
    pub async fn begin_write(&self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
        if self.backend == DatabaseBackend::Sqlite {
            return Ok(self.pool.begin_with("BEGIN IMMEDIATE").await?);
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT id FROM control_state WHERE id=1 FOR UPDATE")
            .fetch_one(&mut *tx)
            .await?;
        Ok(tx)
    }
    pub async fn state_tx(tx: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<(i64, i64)> {
        Ok(
            sqlx::query_as("SELECT active_revision,draft_generation FROM control_state WHERE id=1")
                .fetch_one(&mut **tx)
                .await?,
        )
    }
    pub async fn check_candidate(
        tx: &mut sqlx::Transaction<'_, sqlx::Any>,
        expected: (i64, i64),
    ) -> Result<()> {
        ensure!(
            Self::state_tx(tx).await? == expected,
            "revision conflict: configuration changed during compilation"
        );
        Ok(())
    }
}
