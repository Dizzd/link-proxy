# Configuration Lifecycle

1. Authenticate and validate a draft edit. Serialize mutations with the process mutex.
2. Read the complete draft and compile an immutable candidate off the async runtime.
3. Compare expected_revision with the database's committed pointer. A stale publication returns 409.
4. In one database transaction, insert immutable revision (committed), advance control_state and insert audit event. Import commit also replaces draft, validates base draft_generation and records idempotency result in this transaction.
5. Commit the database transaction before ArcSwap activation. The database and ArcSwap do not form a single cross-system transaction.
6. Activate one complete Arc, update revision/mapping metrics, then acknowledge active/superseded states in the database. Cache notification is asynchronous.

If a process crashes before commit, the previous revision remains authoritative. If it crashes after commit but before swap/acknowledgement, startup reads control_state, verifies checksum, recompiles the committed revision and acknowledges it. A corrupt or invalid committed revision prevents startup rather than silently loading an older revision. Existing running requests retain their old snapshot Arc until completion.

Rollback restores a retained previous revision by compiling it and committing a **new** revision with the previous contents; restore itself does not delete history. A failed compilation does not commit or activate. API draft CRUD does not itself change the active snapshot. Revision 0 is an empty initial configuration. The database stores complete JSON so rollback restores relationships as well as destinations.

Revision cleanup is explicit maintenance: the Revisions page defaults to keeping the newest 100 revisions, previews eligible counts and an oldest-first batch of at most 500, then requires confirmation. Only superseded snapshots outside the newest N are eligible. The runtime snapshot, database pointer and every active/committed status are excluded, including a commit awaiting activation or acknowledgement. Cleanup acquires the mutation lock and a database write transaction, recomputes the preview and rejects stale/tampered input before deleting. Removal and its audit event commit together; draft data, source URLs, stable IDs, import retry results and audit history remain intact. No ArcSwap activation occurs. Each additional batch needs a fresh preview. Removed revisions are unavailable for restore; stored version numbers remain unchanged. Cleanup does not run automatically, compact SQLite files or purge audit/import history.

Optimistic concurrency covers publication and import base revision/generation. Individual draft edits do not require an If-Match header. Each edit captures active revision and draft generation, compiles, then compares both under BEGIN IMMEDIATE (SQLite) or a control_state row lock (PostgreSQL). Stale candidates return conflict before writing. Operate one active control writer per database because import scheduling/recovery is process-local.
