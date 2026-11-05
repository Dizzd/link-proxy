# Revision cleanup verification

Date: 2026-11-05.

## Behavior and safety

The Revisions page offers explicit cleanup with a default of keeping the latest 100 revisions (configurable 1..10000). Preview reports the total stored count, all eligible rows and at most 500 oldest eligible versions. Only superseded revisions outside the newest N are eligible. The actual runtime revision, committed database pointer and all active/committed statuses remain protected. No automatic cleanup runs.

Confirmation submits the complete preview. The application holds its mutation lock, recomputes the preview under SQLite BEGIN IMMEDIATE or a PostgreSQL control_state row lock, and rejects stale/tampered previews with 409 before deletion. Deletion and an audit record commit together. There is no ArcSwap update. Draft data, source URLs, stable IDs, import retry results and audit history remain intact. Each further batch requires a fresh preview. Removed revisions cannot be restored; version numbers are not reused.

Both new API endpoints use existing authenticated administration and session CSRF validation with a 16KiB request limit. No schema migrations or dependency version upgrades were needed; proxy-storage now directly declares the existing workspace serde dependency.

## Executed checks

All commands below completed with exit code 0. Rust/service checks ran on Ubuntu 24.04 WSL with `CARGO_TARGET_DIR=/home/ubuntu/link-proxy-target`; frontend/Chromium checks ran on Windows. Raw logs and screenshots are local and ignored by Git.

| Command | Result | Log |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Passed | `revision-cleanup-fmt.log` |
| `cargo check --workspace --locked` | Passed | `revision-cleanup-check.log` |
| `cargo test --workspace --all-features --locked` | 69 passed, 10 ignored | `revision-cleanup-test.log` |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed | `revision-cleanup-clippy.log` |
| `cargo build --locked -p proxy-server` | Passed | `revision-cleanup-build.log` |
| `python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Five real HTTP tests, process recovery, password reset and graceful shutdown passed | `revision-cleanup-http.log` |
| `python3 scripts/postgres_test.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Disposable PostgreSQL integration test including revision cleanup/import retry, plus five real HTTP tests and recovery passed | `revision-cleanup-postgres.log` |
| `pnpm --dir apps/admin-web build` | TypeScript and Vite build passed after final UI fix | `revision-cleanup-frontend-build.log` |
| `pnpm --dir apps/admin-web test` | 26 passed | `revision-cleanup-frontend-test.log` |
| `pnpm exec playwright test tests/browser/revision-cleanup.spec.ts` | One mocked browser scenario passed | `revision-cleanup-browser.log` |
| `pnpm exec playwright test tests/browser/revision-cleanup-live.spec.ts` | One browser scenario against a real isolated SQLite server passed | `revision-cleanup-live-browser.log` |

Four application tests cover newest-N retention, stale/tampered confirmation rejection, 500-row batches, runtime/database divergence, unacknowledged committed protection, restart recovery, monotonic IDs, retained restore, unchanged draft/source URLs/ArcSwap, audit preservation and idempotent import retry after removal of its old snapshot. Three added API/session tests cover default retention, bounds, missing preview, stale conflict, exact delete count, authentication, CSRF/origin rejection and oversized requests.

The mocked browser scenario covers invalid retention, preview invalidation on input change, cancel without mutation, locked Publish/Restore/input during cleanup, stale conflict, re-preview/retry and no eligible rows. It also verifies that a valid fresh preview survives a delayed refresh of previously stale active-revision data. The real browser scenario signs in, creates a mapping, publishes three revisions, cleans old history, verifies the active configuration is identical, verifies removed history returns 404 on restore and verifies the authenticated audit actor. This live test initially exposed a preview-reset race; after correcting it, both browser scenarios and frontend build/tests passed.

The real browser scenario ran through `scripts/browser_test.py --external` using its temporary settings file and Windows Playwright runner, then returned a zero result to the harness; the isolated server shut down successfully. Mocked UI screenshots at 1440x1000, 375x812 and 812x375 were visually inspected; page-width checks passed with reduced motion. `git diff --check` passed.

## Limits

The workspace run initially skips ten external/scale tests; the HTTP harness explicitly executes five HTTP tests and the disposable PostgreSQL harness executes its PostgreSQL test. Separate Redis and import-scale tests were not run for this change. Browser cleanup used a real SQLite backend; PostgreSQL cleanup was verified through the application integration test. No deployment, Docker execution or production benchmark is claimed. Cleanup does not compact SQLite files, purge audit/import history or automatically remove revisions. Keeping newest N can retain more than N when additional revisions are protected.
