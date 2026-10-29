# Database Schema

SQLx Any selects SQLite or PostgreSQL from the connection URL. Separate embedded migration chains live in migrations/sqlite and migrations/postgres. SQLite migrations 0001..0003 are unchanged; additive 0004_admin_auth.sql adds authentication. PostgreSQL has equivalent entity/revision/import/auth tables, BIGSERIAL revision IDs, JSON text validated via jsonb and indexes for paths/hosts. No automatic data transfer occurs between backends.

| Table | Purpose and relationships |
|---|---|
| domains | ID, unique case-insensitive hostname, enabled, validated entity JSON |
| upstreams | ID, unique name, validated entity JSON |
| mappings | ID, unique mapping_key, status, priority, complete mapping JSON |
| mapping_hosts | mapping FK with cascade, hostname; composite primary key |
| mapping_rules | rule ID, mapping FK cascade, priority/status, schema version, rule JSON |
| config_revisions | monotonic version, committed/active/superseded, checksum, immutable data JSON, created_by |
| control_state | singleton committed revision pointer and draft_generation |
| audit_events | operation, actor, resource_id, revision and timestamp |
| import_jobs | source name/format/policy, lifecycle, base revision/generation, counts, idempotency key, committed revision |
| import_job_errors | job FK, location, field, code, description, suggested correction |
| import_staging_records | job FK, ordinal, stable external ID and serialized staged record |
| admin_users | ID, unique normalized username, salted Argon2id password hash, enabled flag, Unix timestamps |
| admin_sessions | SHA-256 token hash PK, user FK cascade, created/expiry Unix timestamps; raw tokens never stored |

Foreign keys and parameter binding are enabled. SQLite enables WAL/busy timeout and json_valid checks; PostgreSQL validates JSON through jsonb casts. Indexes cover hosts, enabled/priority, path identity, rule parent, revision audits, import job state and staging order. Draft replacement happens inside one transaction. Immutable revisions and audit history survive draft deletion.

The ConfigurationRepository boundary is backed by the runtime-selected Store. SQLite uses BEGIN IMMEDIATE; PostgreSQL serializes control writes with SELECT ... FOR UPDATE on control_state. After full candidate compilation, compare both active_revision and draft_generation under the write lock before committing. Exactly one publisher can accept the same expected revision. Committed revision IDs need not be consecutive after failed PostgreSQL transactions.

Operate one active control-plane process per database: import worker ownership is local and startup marks interrupted parser jobs failed. The SQL locks protect stale cross-process writes but do not provide distributed worker scheduling. PostgreSQL connections should use sslmode=verify-full outside a trusted local test network. Never log database URLs/passwords.
