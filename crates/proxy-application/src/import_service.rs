type ImportJobRow = (
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
    i64,
    i64,
    i64,
    Option<String>,
    Option<i64>,
    String,
);
use crate::ControlPlane;
use anyhow::{Context, Result, ensure};
use proxy_domain::*;
use proxy_import::*;
use proxy_routing::RoutingSnapshot;
use proxy_storage::Store;
use std::sync::Arc;
impl ControlPlane {
    pub async fn submit_import(
        self: &Arc<Self>,
        file_name: String,
        format: Format,
        policy: ConflictPolicy,
        bytes: Vec<u8>,
    ) -> Result<String> {
        let permit = self
            .import_workers
            .clone()
            .try_acquire_owned()
            .context("import worker capacity reached")?;
        ensure!(
            bytes.len() <= self.settings.imports.max_upload_bytes,
            "upload exceeds limit"
        );
        let id = new_id();
        let _guard = self.mutation.lock().await;
        let pending: i64 = sqlx::query_scalar("SELECT count(*) FROM import_jobs WHERE status IN ('pending','validating','ready','committing')").fetch_one(&self.store.pool).await?;
        ensure!(
            pending < self.settings.imports.max_jobs as i64,
            "too many staged import jobs"
        );
        let (revision, generation) = self.store.state().await?;
        let base = self.store.draft().await?;
        sqlx::query("INSERT INTO import_jobs(id,file_name,format,conflict_policy,status,base_revision,base_generation) VALUES($1,$2,$3,$4,'pending',$5,$6)").bind(&id).bind(&file_name).bind(format.as_str()).bind(policy.as_str()).bind(revision).bind(generation).execute(&self.store.pool).await?;
        drop(_guard);
        self.metrics.import_jobs.inc();
        let control = self.clone();
        let job_id = id.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let _timer = control.metrics.import_duration.start_timer();
            if let Err(e) = control
                .validate_import(&job_id, &file_name, format, policy, bytes, base)
                .await
            {
                tracing::warn!(job_id, error = %e, "import validation failed");
                let report = error(&file_name, "file".into(), "invalid_file", e.to_string());
                let _ = control.persist_error(&job_id, &report).await;
                let _ = sqlx::query("UPDATE import_jobs SET status='failed',error_count=error_count+1,updated_at=CAST(CURRENT_TIMESTAMP AS TEXT) WHERE id=$1 AND status NOT IN ('cancelled','completed')").bind(&job_id).execute(&control.store.pool).await;
                control.metrics.import_errors.inc();
            }
        });
        Ok(id)
    }
    async fn persist_error(&self, id: &str, error: &ImportJobError) -> Result<()> {
        sqlx::query("INSERT INTO import_job_errors(job_id,location,field,code,description,suggestion) VALUES($1,$2,$3,$4,$5,$6)").bind(id).bind(&error.location).bind(&error.field).bind(&error.code).bind(&error.description).bind(&error.suggestion).execute(&self.store.pool).await?;
        Ok(())
    }
    async fn validate_import(
        &self,
        id: &str,
        file: &str,
        format: Format,
        policy: ConflictPolicy,
        bytes: Vec<u8>,
        base: Configuration,
    ) -> Result<()> {
        let updated = sqlx::query(
            "UPDATE import_jobs SET status='validating' WHERE id=$1 AND status='pending'",
        )
        .bind(id)
        .execute(&self.store.pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Ok(());
        }
        let limits = self.settings.imports.clone();
        let name = file.to_string();
        let allowlist = self.settings.security.redirect_host_allowlist.clone();
        let (document, errors, summary) = tokio::task::spawn_blocking(move || -> Result<_> {
            let (document, mut errors) = parse(&bytes, format, &name, &limits, &base.upstreams)?;
            let mut summary = ImportJobResult::default();
            match merge(base, &document, policy) {
                Ok((candidate, result)) => {
                    summary = result;
                    if let Err(e) = RoutingSnapshot::compile(0, candidate, &allowlist) {
                        errors.push(error(
                            &name,
                            "configuration".into(),
                            "invalid_configuration",
                            e.to_string(),
                        ));
                    }
                }
                Err(e) => errors.push(error(
                    &name,
                    "configuration".into(),
                    "conflict",
                    e.to_string(),
                )),
            }
            Ok((document, errors, summary))
        })
        .await??;
        self.metrics
            .import_records
            .inc_by(document.mappings.len() as u64);
        self.metrics.import_errors.inc_by(errors.len() as u64);
        let _guard = self.mutation.lock().await;
        let status = self.import_job(id).await?.status;
        if status == "cancelled" {
            return Ok(());
        }
        let mut tx = self.store.pool.begin().await?;
        let mut records = Vec::new();
        for m in document.mappings {
            records.push((m.id.clone(), StagedRecord::Mapping(m)));
        }
        let total = records.len();
        for d in document.domains {
            records.push((format!("domain:{}", d.id), StagedRecord::Domain(d)));
        }
        for u in document.upstreams {
            records.push((format!("upstream:{}", u.id), StagedRecord::Upstream(u)));
        }
        // Each row is durably staged; bounded chunks avoid SQLite parameter limits.
        for (chunk_index, chunk) in records.chunks(100).enumerate() {
            let mut builder = sqlx::QueryBuilder::<sqlx::Any>::new(
                "INSERT INTO import_staging_records(job_id,ordinal,external_id,data) ",
            );
            builder.push_values(
                chunk.iter().enumerate(),
                |mut b, (i, (external, record))| {
                    b.push_bind(id)
                        .push_bind((chunk_index * 100 + i) as i64)
                        .push_bind(external)
                        .push_bind(serde_json::to_string(record).unwrap_or_default());
                },
            );
            let sql = proxy_storage::numbered_parameters(builder.sql());
            let mut built = builder.build();
            let arguments = sqlx::Execute::take_arguments(&mut built)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?
                .context("staging query arguments missing")?;
            sqlx::query_with(&sql, arguments).execute(&mut *tx).await?;
        }
        for e in &errors {
            sqlx::query("INSERT INTO import_job_errors(job_id,location,field,code,description,suggestion) VALUES($1,$2,$3,$4,$5,$6)").bind(id).bind(&e.location).bind(&e.field).bind(&e.code).bind(&e.description).bind(&e.suggestion).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE import_jobs SET status=$1,total_records=$2,valid_records=$3,error_count=$4,result=$5,updated_at=CAST(CURRENT_TIMESTAMP AS TEXT) WHERE id=$6").bind(if errors.is_empty() { "ready" } else { "failed" }).bind(total as i64).bind(if errors.is_empty() { total as i64 } else { 0 }).bind(errors.len() as i64).bind(serde_json::to_string(&summary)?).bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn import_job(&self, id: &str) -> Result<ImportJob> {
        let row: ImportJobRow = sqlx::query_as("SELECT id,file_name,format,conflict_policy,status,base_revision,base_generation,total_records,valid_records,error_count,result,committed_revision,created_at FROM import_jobs WHERE id=$1").bind(id).fetch_one(&self.store.pool).await?;
        Ok(ImportJob {
            id: row.0,
            file_name: row.1,
            format: row.2,
            conflict_policy: row.3,
            status: row.4,
            base_revision: row.5,
            base_generation: row.6,
            total_records: row.7,
            valid_records: row.8,
            error_count: row.9,
            result: row.10,
            committed_revision: row.11,
            created_at: row.12,
        })
    }
    pub async fn staged_document(&self, id: &str) -> Result<ImportDocument> {
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT data FROM import_staging_records WHERE job_id=$1 ORDER BY ordinal",
        )
        .bind(id)
        .fetch_all(&self.store.pool)
        .await?;
        let mut document = ImportDocument {
            mappings: Vec::new(),
            domains: Vec::new(),
            upstreams: Vec::new(),
        };
        for row in rows {
            match serde_json::from_str(&row)? {
                StagedRecord::Mapping(m) => document.mappings.push(m),
                StagedRecord::Domain(d) => document.domains.push(d),
                StagedRecord::Upstream(u) => document.upstreams.push(u),
            }
        }
        Ok(document)
    }
    pub async fn commit_import(
        &self,
        id: &str,
        expected_revision: i64,
        policy: ConflictPolicy,
        confirmation: bool,
        idempotency: &str,
    ) -> Result<ImportJobResult> {
        ensure!(confirmation, "explicit operator confirmation is required");
        ensure!(
            !idempotency.is_empty() && idempotency.len() <= 128,
            "idempotency key required (1..128 bytes)"
        );
        let _guard = self.mutation.lock().await;
        let job = self.import_job(id).await?;
        if job.status == "completed" {
            let original: Option<String> =
                sqlx::query_scalar("SELECT idempotency_key FROM import_jobs WHERE id=$1")
                    .bind(id)
                    .fetch_one(&self.store.pool)
                    .await?;
            ensure!(
                original.as_deref() == Some(idempotency),
                "idempotency key conflict"
            );
            ensure!(
                job.base_revision == expected_revision && job.conflict_policy == policy.as_str(),
                "idempotency request payload differs"
            );
            return Ok(serde_json::from_str(
                job.result.as_deref().context("missing commit result")?,
            )?);
        }
        ensure!(job.status == "ready", "import is not ready");
        ensure!(
            job.conflict_policy == policy.as_str(),
            "commit conflict_policy must match validated policy"
        );
        let (revision, generation) = self.store.state().await?;
        ensure!(
            revision == expected_revision
                && revision == job.base_revision
                && generation == job.base_generation,
            "revision conflict: configuration changed since validation; re-upload and validate"
        );
        let document = self.staged_document(id).await?;
        let (candidate, mut result) = merge(self.store.draft().await?, &document, policy)?;
        let snapshot = self.compile(0, candidate.clone()).await?;
        let mut tx = self.store.begin_write().await?;
        Store::check_candidate(&mut tx, (revision, generation)).await?;
        sqlx::query("UPDATE import_jobs SET status='committing',idempotency_key=$1 WHERE id=$2")
            .bind(idempotency)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        Store::write_draft(&mut tx, &candidate).await?;
        let version =
            Store::commit_revision(&mut tx, &candidate, expected_revision, "import_commit").await?;
        result.revision = Some(version);
        sqlx::query("UPDATE import_jobs SET status='completed',committed_revision=$1,result=$2,updated_at=CAST(CURRENT_TIMESTAMP AS TEXT) WHERE id=$3").bind(version).bind(serde_json::to_string(&result)?).bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        self.activate(snapshot, version).await;
        Ok(result)
    }
    pub async fn cancel_import(&self, id: &str) -> Result<()> {
        let _guard = self.mutation.lock().await;
        let mut tx = self.store.pool.begin().await?;
        let result = sqlx::query("UPDATE import_jobs SET status='cancelled',updated_at=CAST(CURRENT_TIMESTAMP AS TEXT) WHERE id=$1 AND status IN ('pending','validating','ready')").bind(id).execute(&mut *tx).await?;
        ensure!(
            result.rows_affected() == 1,
            "job cannot be cancelled after activation or failure"
        );
        sqlx::query("DELETE FROM import_staging_records WHERE job_id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
