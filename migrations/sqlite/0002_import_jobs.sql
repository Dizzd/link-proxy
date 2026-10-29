ALTER TABLE audit_events ADD COLUMN actor TEXT NOT NULL DEFAULT 'admin-token';
ALTER TABLE audit_events ADD COLUMN resource_id TEXT;
ALTER TABLE config_revisions ADD COLUMN created_by TEXT NOT NULL DEFAULT 'admin-token';
CREATE INDEX IF NOT EXISTS idx_import_staging_job ON import_staging_records(job_id,ordinal);
