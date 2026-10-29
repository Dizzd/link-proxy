CREATE TABLE IF NOT EXISTS domains (
    id TEXT PRIMARY KEY, hostname TEXT NOT NULL UNIQUE, enabled BIGINT NOT NULL,
    data TEXT NOT NULL CHECK (jsonb_typeof(data::jsonb) IS NOT NULL)
);
CREATE TABLE IF NOT EXISTS upstreams (
    id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, data TEXT NOT NULL CHECK (jsonb_typeof(data::jsonb) IS NOT NULL)
);
CREATE TABLE IF NOT EXISTS mappings (
    id TEXT PRIMARY KEY, mapping_key TEXT NOT NULL UNIQUE, enabled BIGINT NOT NULL,
    priority BIGINT NOT NULL, data TEXT NOT NULL CHECK (jsonb_typeof(data::jsonb) IS NOT NULL)
);
CREATE TABLE IF NOT EXISTS mapping_hosts (
    mapping_id TEXT NOT NULL REFERENCES mappings(id) ON DELETE CASCADE, hostname TEXT NOT NULL,
    PRIMARY KEY(mapping_id,hostname)
);
CREATE INDEX IF NOT EXISTS idx_mapping_hosts_hostname ON mapping_hosts(hostname);
CREATE INDEX IF NOT EXISTS idx_mappings_enabled_priority ON mappings(enabled,priority DESC);
CREATE TABLE IF NOT EXISTS mapping_rules (
    id TEXT PRIMARY KEY, mapping_id TEXT NOT NULL REFERENCES mappings(id) ON DELETE CASCADE,
    priority BIGINT NOT NULL, enabled BIGINT NOT NULL, condition_schema_version BIGINT NOT NULL DEFAULT 1,
    data TEXT NOT NULL CHECK (jsonb_typeof(data::jsonb) IS NOT NULL)
);
CREATE INDEX IF NOT EXISTS idx_mapping_rules_mapping ON mapping_rules(mapping_id,priority DESC);
CREATE TABLE IF NOT EXISTS config_revisions (
    version BIGSERIAL PRIMARY KEY, status TEXT NOT NULL CHECK(status IN ('committed','active','superseded')),
    checksum TEXT NOT NULL, data TEXT NOT NULL CHECK (jsonb_typeof(data::jsonb) IS NOT NULL), created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::TEXT)
);
CREATE TABLE IF NOT EXISTS control_state (
    id BIGINT PRIMARY KEY CHECK(id=1), active_revision BIGINT NOT NULL DEFAULT 0, draft_generation BIGINT NOT NULL DEFAULT 0
);
INSERT INTO control_state(id) VALUES(1) ON CONFLICT(id) DO NOTHING;
CREATE TABLE IF NOT EXISTS audit_events (
    id BIGSERIAL PRIMARY KEY, operation TEXT NOT NULL, revision BIGINT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::TEXT)
);
CREATE TABLE IF NOT EXISTS import_jobs (
    id TEXT PRIMARY KEY, file_name TEXT NOT NULL, format TEXT NOT NULL, conflict_policy TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending','validating','ready','committing','completed','failed','cancelled')),
    base_revision BIGINT NOT NULL, base_generation BIGINT NOT NULL, total_records BIGINT NOT NULL DEFAULT 0,
    valid_records BIGINT NOT NULL DEFAULT 0, error_count BIGINT NOT NULL DEFAULT 0,
    result TEXT, idempotency_key TEXT UNIQUE, committed_revision BIGINT,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::TEXT), updated_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::TEXT)
);
CREATE INDEX IF NOT EXISTS idx_import_jobs_status_created ON import_jobs(status,created_at);
CREATE TABLE IF NOT EXISTS import_job_errors (
    id BIGSERIAL PRIMARY KEY, job_id TEXT NOT NULL REFERENCES import_jobs(id) ON DELETE CASCADE,
    location TEXT NOT NULL, field TEXT NOT NULL, code TEXT NOT NULL, description TEXT NOT NULL, suggestion TEXT
);
CREATE INDEX IF NOT EXISTS idx_import_errors_job ON import_job_errors(job_id,id);
CREATE TABLE IF NOT EXISTS import_staging_records (
    job_id TEXT NOT NULL REFERENCES import_jobs(id) ON DELETE CASCADE, ordinal BIGINT NOT NULL,
    external_id TEXT NOT NULL, data TEXT NOT NULL CHECK (jsonb_typeof(data::jsonb) IS NOT NULL), PRIMARY KEY(job_id,ordinal), UNIQUE(job_id,external_id)
);

CREATE UNIQUE INDEX idx_domains_hostname_ci ON domains(lower(hostname));
