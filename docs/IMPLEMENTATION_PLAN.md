# Dynamic URL Proxy Implementation Plan and Evidence

**Goal:** Deliver a persistent unified mapping platform with a Pingora data plane and authenticated Axum control plane.
**Architecture:** Domain -> routing/storage/cache/import -> application -> admin API/Pingora -> binary. Compile immutable routing snapshots before database revision commit, then activate with ArcSwap. The selected SQLite/PostgreSQL database is authoritative.
**Stack:** Rust stable 1.98.1 tested, Pingora 0.9, Tokio 1.53.2 pinned, Axum 0.8, SQLx 0.8, React 19, TypeScript, Vite 7, pnpm.

## Initial inspection
2026-10-29: Existing uncommitted single-crate Rust sources, migration and tests were found. They were migrated into the requested workspace. Temporary compatibility layers were later removed during repository cleanup. Existing SQLite migration 0001 was preserved verbatim in the embedded migrations/sqlite chain. No existing database or active business routes were modified.

## Phase status
- [x] Phase 1: workspace with all ten library/test crates, proxy-server binary, pnpm workspace, basic CI and technical documentation. Cargo metadata and workspace compilation executed.
- [x] Phase 2: framework-independent model, exact/template/prefix indexes, mapping-key template index, deterministic priority/conflict validation, bounded nested conditions, UTC clocks, timezones/DST regression, matching/rewrite/snapshot tests executed.
- [x] Phase 3: actual Pingora HTTP redirect and streamed reverse proxy, template rewrite, Host/forwarding policy, connection-time CIDR validation, size limits, 404/405 and shutdown. Five real HTTP integration binaries executed.
- [x] Phase 4: three SQLx migrations, authenticated CRUD/rules/domains/upstreams, filtering/pagination, validation/simulation, structured errors, rate limit and transactional actor/resource audit. API tests executed.
- [x] Phase 5: compile/commit/activate protocol, expected revisions, rollback as new revision, startup checksum recovery and old-Arc isolation. Persistent restart and live publish/restore executed.
- [x] Phase 6: memory mode independent of Redis; optional versioned envelopes, monotonic Redis updates, bounded Pub/Sub hints, periodic authoritative reconciliation and failure retention. Three disposable real Redis tests executed, including server shutdown.
- [x] Phase 7: bounded uploads and CSV/XLSX/JSON parsers/exports, four-sheet XLSX relationships, durable staging, preview, explicit idempotent commit, cancellation and error reports. Round trips and 1k/10k/50k scale imports executed.
- [x] Phase 8: ten React pages, mapping/rule/action editors, nested conditions, simulator, import/preview/commit/error downloads, export, revisions/cache/metrics. Frontend production build, seven Vitest tests and one full real-API Chromium workflow executed. Desktop/mobile screenshots inspected; no document horizontal overflow at 390 px.
- [ ] Phase 9 final deployment gate: security checks, metrics, release Criterion benchmarks, memory measurement, Docker/Compose/Caddy configuration and documentation completed. **Docker build/run is unverified because no Docker engine/CLI is installed.** This phase and complete-product acceptance remain open until actual container verification.

## Executed verification
Raw output is in docs/verification/. Mandatory commands executed on Linux/WSL for Rust and Windows/Node for frontend:

```sh
cargo fmt --all -- --check
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build -p proxy-server
pnpm --dir apps/admin-web build
pnpm --dir apps/admin-web test
python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server
python3 scripts/redis_test.py
cargo test -p proxy-application --test imports import_large_10000_and_50000_records -- --ignored --nocapture
cargo bench -p proxy-routing --bench routing_bench
cargo bench -p proxy-import --bench import_bench
cargo run --release -p proxy-routing --example memory
```

Chromium was run against an isolated live memory database with the production UI served by the binary. `node node_modules/@playwright/test/cli.js test` returned exit code 0, one workflow passed, and generated verification/admin-desktop.png and admin-mobile.png. That historical run used the original API token UI; the updated password/session workflow is verified separately below.

## Measured results
See BENCHMARK.md for full intervals and limitations. At 100k mappings: exact lookup 168.71 ns, snapshot compilation 322.42 ms, process RSS 148,880 kB. Nested condition evaluation 277.47 ns. CSV parse and per-record validation at 50k: 66.575 ms / 0.75104 million records/s. Separate debug SQLite-memory scale workflow: 10k in 4.645334636 s and 50k in 22.385757377 s, each publishing one revision.

## Acceptance record
Verified by tests: initialized/buildable workspace; Pingora/Axum listeners; proxy/static/conditional redirects; host sharing; template parameters; runtime publication; persistent restart recovery; memory-only operation; optional Redis and failure retention; CSV/XLSX/JSON import; dry-run/preview/errors; one-revision explicit import commit; export; rollback; authenticated administration; real browser UI; automated security checks; Rust and frontend tests/builds; documented startup and real example files.

Unverified: Docker image build/run/health/volume behavior, public TLS gateway integration, production network throughput/p95/p99 and load endurance. No such criterion is marked passed.

## Engineering limits
- One control writer per SQLite database; Redis does not replicate independent SQLite databases. PostgreSQL is now implemented (see the dated extension below).
- Draft edits are process-serialized; optimistic concurrency is enforced for publication/import, not per-field If-Match on individual draft edits.
- Username/password sessions replace the original token UI; multiuser RBAC remains outside scope. Audit records retain actor, operation, resource ID and revision.
- CSV formula-safety escaping can affect display values; use JSON/XLSX for lossless entity interchange. Import parsing is bounded-memory by configured file/decompression limits and worker count, not constant-memory streaming.
- Request/body limits and upstream connect/read/write timeouts exist; a total downstream body wall-clock deadline and trusted ingress TLS scheme policy remain hardening work. Multi-hop redirect loops remain an operator responsibility.
- A reproducible Pingora 0.9 / Tokio 1.54 buffer panic was found by actual HTTP testing; Tokio is pinned to 1.53.2 until upstream compatibility is verified.

Original duplicate implementations and temporary compatibility layers were removed after migration. Root is a virtual Cargo workspace; source and Rust integration tests live in their owning crates. No placeholder transport, fake endpoint or separate QR evaluator is used.

## Repository cleanup, 2026-10-29
- Removed temporary admin/ and root src/ compatibility layers; root Cargo.toml is now a virtual workspace with eleven member packages.
- Moved routing integration tests into proxy-routing, import/cache application tests into proxy-application and authenticated API tests into proxy-admin-api. Tests import their actual dependencies directly.
- Updated Cargo/pnpm lockfiles, canonical frontend commands, CI, Docker build inputs and test scripts. Redis-specific application tests use the explicit redis-cache feature.
- Removed generated Playwright test-results/ and Python __pycache__/ directories, and the unused root migration copy. Embedded migrations/sqlite/0001_initial.sql was left byte-for-byte unchanged (SHA256 5cb12049204e8514a8767bbab502547919b716506a1324ededf50c2bce19ff69).
- Retained installed node_modules and frontend dist assets for local build/runtime, browser tests/fixtures and requested documentation/deployment/examples.
- Cleanup verification is recorded separately in docs/verification/cleanup-*.log; historical implementation measurements remain unchanged.

## PostgreSQL and username/password administration plan

Goal: keep existing SQLite installations working while adding an explicitly selected PostgreSQL backend and persistent browser sessions.
Architecture: SQLx 0.8 runtime-selected Any pool with backend-specific additive migration chains; portable bound SQL and database write locks protect revisions. The configured database is authoritative; routing remains on immutable ArcSwap snapshots. Argon2id password hashes and SHA-256 session-token hashes are persisted; browser sessions use HttpOnly SameSite=Strict cookies plus CSRF headers. API tokens are optional and disabled in server configuration by default.

- [x] Storage: enable SQLx postgres/any, retain original SQLite migration bytes, add SQLite auth migration and PostgreSQL schema/auth migrations. Convert repositories/import staging/management queries to portable parameters and returning revision IDs. Verify existing SQLite tests, a real PostgreSQL workflow and crash/revision recovery.
- [x] Transactions: lock the singleton control row for database writes and compare candidate draft generation/revision after acquiring the lock. Compile before commit and activate only after commit. Test independent writers cannot publish stale candidates.
- [x] Authentication: add auth users/sessions repositories; bootstrap the first administrator from ADMIN_USERNAME/ADMIN_PASSWORD without overwriting an existing account. Add login/session/logout/password endpoints, bounded Argon2 workers, login throttling, session expiration/revocation, CSRF and cookie controls. Bind audit actor to the authenticated user.
- [x] UI: replace token login with username/password, restore sessions after reload, send cookie/CSRF on API calls, sign out and change password. Update unit tests and the real Chromium workflow.
- [x] Deployment/docs: add PostgreSQL config/Compose overlay, environment examples and SQLx database selection status; document bootstrap/reset, optional API tokens and existing SQLite data preservation.
- [x] Verification: cargo fmt/check/test/clippy on Linux; real SQLite and PostgreSQL HTTP/auth/import/revision workflows; real Redis regression; frontend build/Vitest/Chromium. Record actual results under docs/verification without claiming unexecuted deployment checks.

### Extension verification results (2026-10-29)

- Linux/WSL cargo fmt --all -- --check, cargo check --workspace --locked, cargo test --workspace --all-features --locked and cargo clippy --workspace --all-targets --all-features --locked -- -D warnings returned success. Workspace tests: 56 passed, 10 deliberately ignored (external services/scale); see password-{fmt,check,workspace-tests,clippy}.log.
- Eight session/auth API tests pass, including Secure cookies and structured bounded malformed login input. An additive SQLite upgrade test verifies original migration checksums, retained mapping IDs/source URLs/configuration/revision history and an empty new user table.
- Real PostgreSQL 16 disposable-cluster integration passes migration, draft CRUD, import staging/explicit idempotent commit, CSV/XLSX/JSON export, rollback, restart recovery and stale concurrent publisher rejection. See postgres-integration.log.
- Five real HTTP tests pass on SQLite and the same five pass on PostgreSQL, with real Pingora reverse proxy/redirect/conditional routing/dynamic publish/import. Both harnesses also pass actual process restart and --reset-admin-password CLI execution without changing committed routing revision. See password-http-sqlite.log and postgres-integration.log.
- Three real Redis availability/notification/failure regression tests pass; active traffic keeps its local snapshot. See password-redis.log.
- Frontend production build and nine Vitest tests pass. Chromium completes the full real application workflow once on SQLite and once on PostgreSQL, including username/password login, cookie flags, reload persistence, settings/database status, password change, old-password rejection, signout, mapping publish/simulation/import/export and responsive layout. See password-ui-{build,tests}.log and password-browser-{sqlite,postgres}.log. Minor subsequent frontend changes only renamed state setters; build/Vitest were rerun.
- Original embedded SQLite 0001 SHA256 remains 5cb12049204e8514a8767bbab502547919b716506a1324ededf50c2bce19ff69; migrations 0001..0003 were not edited.
- Deployment files/Compose PostgreSQL overlay and documentation are updated. Docker is unavailable on this host, so container build/runtime remains unverified. No performance benchmark was rerun for this extension; historical benchmark results retain their original scope.
- Operational limits: one active control-plane process per database (import ownership/recovery is local), no automatic SQLite-to-PostgreSQL data migration, no multiuser/RBAC management UI. Existing SQLite installations remain the default until DATABASE_URL/database_url explicitly selects PostgreSQL.

## GitHub container release and deployment bundle plan

Goal: publish a versioned GHCR image and attach a self-contained deployment bundle to each published GitHub Release. Servers need only Docker Compose and edited environment/configuration files.

Architecture: reuse the Rust/frontend CI workflows as release gates; build linux/amd64 with the existing multi-stage Dockerfile, smoke-test the local image, push a release tag to GHCR, and pin the deployment bundle to the returned image digest. Generate deployment assets in a temporary/output directory without copying source code or secrets. Root Compose remains the source-build workflow; deploy/docker-compose.yml contains image-only runtime configuration. Both now declare the complete PostgreSQL/proxy/Caddy/Redis stack with profiles in one file.

- [x] Add workflow_call triggers to .github/workflows/{rust-ci,frontend-ci}.yml; add .github/workflows/release.yml with least-privilege job permissions, release tag validation, GHCR login, Buildx caching, image smoke test and release asset upload.
- [x] Add deploy/docker-compose.yml and deploy/.env.example, plus scripts/package_release.py. Consolidate and remove PostgreSQL/Redis overlays; package the single full-stack Compose, Caddyfile and Docker TOML configuration, a runtime README, release metadata and checksum file; substitute the lowercased repository/image digest and reject malformed metadata/output conflicts.
- [x] Add focused packaging tests and CI deployment validation for SQLite, PostgreSQL and PostgreSQL+Redis using docker compose config --quiet with dummy test credentials. Verify image startup/session login using the actual built container in the release workflow.
- [x] Document release creation, GHCR visibility/private authentication, exact server download/start/update commands, persistent volume naming and optional SQLite/Redis configuration. Record executed local checks separately from GitHub/Docker checks that require those runtimes.

### Unified stack and release verification (2026-10-29)

- User requested a single complete service stack. Root docker-compose.yml and release deploy/docker-compose.yml now each define proxy, PostgreSQL, Caddy and Redis. Removed docker-compose.postgres.yml/docker-compose.redis.yml. Example environment enables postgres,caddy; Redis is optional, and SQLite/external DB/ingress are selected by profiles without combining files. Existing data is retained by keeping the original Compose project name and volumes; selecting another DB does not copy data.
- scripts/package_release.py emits the tar archive and loose runtime assets only, pins the lowercased repository's GHCR digest, includes release metadata and SHA256SUMS, rejects unsafe/unpinned metadata and refuses nonempty output directories. It never packages .env or source.
- Workflow .github/workflows/release.yml triggers on published releases, validates tags, reuses Rust/frontend/deployment CI, builds linux/amd64, smoke-tests the exact local image through PostgreSQL/Redis/Caddy, pushes that tested image, generates the digest-pinned bundle and uploads assets with GITHUB_TOKEN. No production credentials are read by GitHub Actions. Release smoke fixtures use random credentials/project names and remove only their own disposable volumes.
- Executed locally: three packaging integrity/security tests; real Docker Compose 2.39.4 parsing for four service combinations each in release/source (eight total), including the actual environment example interpolation and all bind-mount files; actionlint 1.7.12 across all workflows; Python syntax compilation; Linux cargo fmt/checks via fmt, all-feature workspace tests (56 passed, 10 external/scale tests ignored), all-target/all-feature clippy; frontend production build. Evidence is in docs/verification/release-*.log.
- Docker Engine is unavailable locally. Image build, full-stack container smoke, GHCR push and Release upload have NOT been executed; their checks are implemented in the workflow and must succeed on GitHub before a release is considered published/verified. No new release was created or pushed by this local task. The image currently supports linux/amd64 only. Public pull access requires public GHCR visibility; private images require read:packages authentication.

## v0.1.0 pre-release publication

- Published the initial v0.1.0 pre-release with updated private-repository download/GHCR instructions and explicit versioned asset URLs. Release validation passed Rust, frontend and deployment jobs on GitHub.
- The first actual Docker build found a runtime account collision: Debian already contains the system user proxy. Corrected the Dockerfile to create a separate urlproxy account with UID 10001 and a nologin shell.
- Added workflow_dispatch recovery restricted to the exact release tag, keeping reusable validation and image source at the same Git commit. Existing releases must exist before publishing assets.
- Image build/smoke/push/assets remain pending until a successful recovery run. No image or deployment assets were uploaded by the failed run.

### v0.1.0 publication completed

Recovery run https://github.com/Dizzd/link-proxy/actions/runs/40891455791 succeeded from release commit 21197a0697bc880eebb8f1ffc06946a522dda9fe. It built the image and passed the actual container smoke test covering the React UI, password session, PostgreSQL, Redis, publication and Caddy -> Pingora redirect, then pushed GHCR and uploaded nine release assets. All Compose combinations passed again on the hosted runner.

Published image: ghcr.io/dizzd/link-proxy:v0.1.0, pinned manifest sha256:a319e587c99eeca6b8fa452e35e905fe1d566608c8c74b4ee77a5422d68c258d. Downloaded the real release archive and SHA256SUMS using authenticated GitHub access: all eight checksums and seven archive members matched, and release.json/.env.example referenced the pushed digest. Evidence: docs/verification/release-v0.1.0-verification.log. GitHub names the standalone environment asset default.env.example; the archive preserves .env.example. The completed release remains a pre-release, not Latest. The tag remains at the verified image source commit; subsequent documentation-only commits do not retarget published artifacts.

## Single runtime configuration plan

Use one deployed config.toml for all cache modes. CACHE_MODE overrides only cache.mode and cache.redis.enabled at startup; all memory snapshots, database authority, limits and Redis tuning remain unchanged. Missing CACHE_MODE preserves TOML behavior, while unknown/empty/non-Unicode overrides fail startup.

- [x] Implement and unit-test cache mode override parsing in proxy-cache, including reverse overrides, preserved Redis tuning and invalid values.
- [x] Remove config/config.redis.toml and its bind mounts; set CACHE_MODE in source/release Compose and environment examples, with one fixed /app/config.toml.
- [x] Update release packaging/profile validation/full-stack smoke and add real binary startup checks using the same file in memory/Redis modes. Update CI to run those checks.
- [x] Update README/deployment/cache documentation with the single-file flow and old v0.1.0 compatibility. Verify fmt/check/test/clippy, frontend build, packaging, Compose, and HTTP/Redis regressions. Published v0.1.0 artifacts remain unchanged; new environment support requires a new image.

### Single-file configuration verification

- Removed mode-specific TOML files and binds from source/release Compose and the package manifest; deployment uses only /app/config.toml. CACHE_MODE defaults to memory and enables/disables Redis consistently when explicitly set. TOML tuning is retained, and absent overrides preserve legacy TOML behavior.
- Added three cache configuration unit tests and a real binary startup fixture. The same unchanged TOML passed unset, memory_redis with an owned Redis daemon, and memory with unreachable Redis; invalid, empty and non-UTF-8 environment values were rejected before startup. No process-global environment mutation or unsafe code was added to tests.
- Executed Linux cargo fmt/check, all-feature workspace tests (59 passed, 10 external/scale tests ignored), clippy with all targets/features and -D warnings, and proxy-server build. All passed. Production frontend build passed; no UI behavior changed.
- Three packaging tests and eight real Compose CLI combinations (source/release, SQLite-only, SQLite+Caddy, PostgreSQL+Caddy and PostgreSQL+Caddy+Redis) passed, asserting one TOML mount, a fixed PROXY_CONFIG and the selected CACHE_MODE. actionlint and Python compilation passed. The future image smoke now uses CACHE_MODE with the single TOML file.
- Re-ran five actual Pingora HTTP tests each on SQLite and PostgreSQL, PostgreSQL migration/import/concurrency/recovery integration, both process-restart/password-reset flows and three real Redis notification/failure tests. All passed. Evidence: docs/verification/release-single-config-*.log.
- New container image build/publication is not claimed: the current v0.1.0 release and pinned image remain unchanged. CACHE_MODE requires a newly built image; legacy images still support one TOML when its mode/enabled fields are set directly. Migration documentation preserves the effective old settings, data volumes and project name. Existing SQL migration bytes were not modified.

## v0.2.0 pre-release preparation

Release scope: one runtime config.toml, CACHE_MODE startup overrides, automatic Redis enable/disable, updated Compose/runtime archive and migration guidance from v0.1.0. Bump all workspace packages to 0.2.0 via the shared workspace version and regenerate Cargo.lock. Keep schema versions, data and the published v0.1.0 tag/image/assets unchanged. Publish v0.2.0 as a pre-release (not Latest), then verify the hosted image/full-stack smoke/GHCR/assets and downloaded checksums before marking publication complete.

## Live cache badge correction

The sidebar's Memory snapshots text was static and did not identify configuration-cache mode or Redis connectivity. Replace it with an authenticated cache-status query shared with Settings. Show actual mode/connection, explicit initial/unknown states, and never display stale Redis-connected data after a failed refresh. Routing continues to use its in-memory snapshot in both modes.

- [x] Add a typed cache-status API and shared TanStack Query hook; replace the static sidebar label with an accessible status badge.
- [x] Reuse the query in Settings and clarify configuration synchronization versus request routing.
- [x] Verify memory/Redis/loading/failure/recovery states with focused UI tests and browser checks, plus frontend build. Record actual validation and keep published releases unchanged.

### Cache badge verification

- Replaced the static sidebar label with a typed authenticated query shared by Settings, polling every five seconds. Actual memory/Redis mode and reported connectivity are separate; initial, unknown/error and not-yet-synchronized states are explicit. Failed API refreshes suppress stale connected claims. An atomic contextual status region announces changes without moving focus; status is conveyed in text.
- Added eight focused presentation tests, bringing Vitest to 17 passing tests. Production frontend build passes. Chromium passed two workflows: the real memory-mode administration/auth/publish/import/export flow and isolated API fixtures exercising loading, Redis connected, API failure with stale data, Redis unavailable, recovery and memory-only transition. Redis fixture responses are test-only and do not reconfigure a production server. Responsive width assertions and desktop/mobile screenshots passed; the desktop screenshot was visually inspected.
- Linux cargo fmt, all-feature workspace tests (59 passed), all-target/all-feature clippy -D warnings and proxy-server build passed with unchanged backend logic. Evidence: docs/verification/release-cache-badge-*.log and updated administration screenshots.
- The initial browser retry found ambiguous old status selectors after adding a second legitimate live region; selectors were scoped to the main content and both workflows passed on a fresh isolated backend. No backend configuration, persistence, routing hot path or published v0.2.0 image/tag was changed. Deployment needs a newly built frontend image to display the corrected badge.

## Grouped navigation and visual monitoring

- [x] Group existing sidebar URLs into Routing, Data management and Operations. Provide keyboard-operable collapse controls, expanded state and an indicator when a collapsed group contains the current page; retain Overview as a direct link.
- [x] Replace the raw-only Monitoring view with request/mapping summary cards, request-rate and interval-average response-time charts, import totals, optional exact sample table and collapsed raw Prometheus output. Keep authenticated polling at five seconds with pause/resume and explicit refresh-failure state.
- [x] Keep history local to the open page and bounded to 60 samples. Derive rates using actual elapsed time, distinguish zero traffic from unavailable latency, and break trends on counter resets or long gaps. Do not imply server-side historical retention or change request-time routing/persistence.
- [x] Execute frontend tests/build, Linux Rust checks and real HTTP/process-recovery harness. Verify desktop/mobile charts, grouped navigation, pause/resume and refresh-error recovery in Chromium. Evidence: docs/verification/release-monitoring-*.log and monitoring-desktop.png / monitoring-mobile.png.

Verification: 22 frontend tests passed; all three Chromium workflows passed, including the existing real administration and cache-status flows. Linux cargo fmt/check/test/clippy and proxy-server build passed; workspace tests reported 59 passed and 10 ignored. The actual HTTP harness explicitly ran the five Pingora HTTP scenarios and process recovery/password reset; the remaining ignored external/scale tests were not executed by this UI task. No Docker runtime, deployment, release publication or production benchmark is claimed. The browser fixtures are isolated to their test context. The final mobile chart uses measured container width so axis labels stay readable. Published images require rebuilding to include the new frontend.

## v0.3.0 pre-release preparation

Release scope: grouped/collapsible sidebar navigation, visual Monitoring charts and summaries, and live configuration-cache/Redis status since v0.2.0. Bump the shared Rust workspace version to 0.3.0 and regenerate Cargo.lock without dependency upgrades. Keep existing SQL migrations, data, source URLs, runtime configuration and published releases unchanged. Publish v0.3.0 as a pre-release, not Latest. Hosted image build, full-stack smoke, GHCR push, asset upload and downloaded bundle checks remain pending until execution evidence confirms them.

### v0.3.0 publication verified

Published https://github.com/Dizzd/link-proxy/releases/tag/v0.3.0 as a pre-release, not Latest, from commit 280940bd112a2646af5b26315934ff26d8b3e41f. Container release run https://github.com/Dizzd/link-proxy/actions/runs/41023415399 completed successfully: Rust/frontend/deployment validation, linux/amd64 image build, actual PostgreSQL/proxy/Caddy/Redis smoke test, GHCR push, Compose validation and upload of all eight assets.

Image ghcr.io/dizzd/link-proxy:v0.3.0 has manifest digest sha256:581fcacc6128774450d2fd80adc9dbece61d3002a3ee263486f066e32466021e. Downloaded all actual release assets via authenticated GitHub access: all seven SHA256SUMS entries and six archive members matched; release.json and the environment example pin that image digest. Evidence is in docs/verification/release-v0.3.0-assets.log, release-v0.3.0-hosted.log and release-v0.3.0-workflow.json. The image targets linux/amd64; production deployment/benchmarks are not claimed. This documentation update does not retarget the published tag or modify existing releases.

## Optional mapping IDs in bulk import

- [x] Allow missing/empty external_id for CSV/XLSX mappings, generating UUIDs for new flat rows while preserving explicit IDs and existing mapping_json identities. Keep Rules references and key-conflict policies explicit.
- [x] Verify generated IDs persist through staging, explicit commit and idempotent retry; a new upload with an existing key cannot silently overwrite the mapping.
- [x] Execute Linux fmt/check/workspace all-feature tests/Clippy/server build and the real HTTP/recovery/shutdown harness; execute frontend build and tests. Workspace: 62 passed, 10 ignored; frontend: 22 passed. The HTTP harness explicitly executed its five ignored tests. Other ignored external/scale tests were not run for this fix.

Evidence: [bulk-import-id.md](verification/bulk-import-id.md) and bulk-import-id-*.log. Parsing the original template.xlsx generated all seven IDs and exposed six missing-host errors (Mappings rows 3-8); the workbook was preserved. Deployment and production benchmarks are not claimed.

## v0.3.1 pre-release preparation

Include the optional bulk-import mapping ID fix and verification-artifact ignore rules. Bump workspace packages and Cargo.lock to 0.3.1 without upgrading dependencies; update current release instructions. Three release-packaging tests passed locally. The bulk-import validation evidence above covers the functional fix. Hosted tag validation, image build, full-stack smoke, GHCR publication and deployment asset download verification remain pending until the release workflow executes successfully. Keep this release marked as a pre-release, not Latest.

### v0.3.1 publication verified

Published https://github.com/Dizzd/link-proxy/releases/tag/v0.3.1 as a pre-release, not Latest, from 71bf8c53e41cef3b72168bbe38fc854e6ffeebde. Workflow https://github.com/Dizzd/link-proxy/actions/runs/41032028403 passed all four jobs, including real Rust HTTP/PostgreSQL/configuration tests, frontend/deployment validation, image build, PostgreSQL/proxy/Caddy/Redis smoke, GHCR push and all eight asset uploads.

Downloaded and verified all seven checksums and all six archive members. The published image digest matches release.json and the environment example: ghcr.io/dizzd/link-proxy@sha256:20df0b24f96c5d3b9ed67b394daa83ccad3991b35e1d2021b5923317233fc9bb (linux/amd64). Evidence: [release-v0.3.1.md](verification/release-v0.3.1.md). Raw logs and downloads remain local under the updated ignore policy. Production deployment and production benchmarks are not claimed.

## v1.0.0 README preparation

Updated the root README to target v1.0.0 deployment and workflow recovery, retain historical verification links and document stable release preparation and operational limits. Source code and package versions remain unchanged. Cargo is still 0.3.1 and the private frontend package is 0.1.0; align them before tagging.

Executed Linux/WSL fmt/check/test/clippy, server build and the real HTTP/recovery/shutdown harness; all passed. Workspace tests reported 62 passed and 10 ignored; the HTTP harness explicitly ran five ignored HTTP tests. Frontend build and 22 tests passed. Evidence and unexecuted checks: [readme-v1.0.0.md](verification/readme-v1.0.0.md). No v1.0.0 tag, GitHub Release, Docker publication, deployment or benchmark is claimed.

## v1.0.0 stable release preparation

Aligned Cargo workspace packages and the private frontend to 1.0.0, regenerated Cargo.lock offline and confirmed only eleven internal package versions changed. Linux/WSL fmt/check/test/clippy, server build, real HTTP/recovery/shutdown, disposable PostgreSQL tests, runtime Redis configuration checks and packaging tests passed. Frontend build and 22 tests passed. See [release-v1.0.0.md](verification/release-v1.0.0.md) for commands, logs and unexecuted checks. Hosted image smoke/publication and downloaded-asset verification remain pending at source commit time.

### v1.0.0 publication verified

Published https://github.com/Dizzd/link-proxy/releases/tag/v1.0.0 as a stable release marked Latest, from commit 20df8f041022a4c2c65f84f9141c344a58c72d91. Workflow https://github.com/Dizzd/link-proxy/actions/runs/41383412049 passed all four jobs, including actual image build, PostgreSQL/proxy/Caddy/Redis smoke, GHCR push and upload of all eight assets. Download verification passed all seven checksums and six archive-member comparisons. The pinned linux/amd64 image digest is sha256:60e6bbefabab2172cd876e42d6f94f66ae274ea94fa70d621f6cac666526166a, matching metadata, environment and hosted push logs. Evidence: [release-v1.0.0.md](verification/release-v1.0.0.md). No production deployment or production benchmark is claimed.

## Mapping bulk deletion

- [x] Add page-scoped mapping selection (up to 25 rows), mixed-state select-all, clear selection and confirmed draft deletion. Clear selection on filter/page changes and disable controls during deletion.
- [x] Reuse authenticated DELETE requests sequentially, stop on failure, report partial completion, refresh and retain remaining visible selections. Recover valid pagination after deleting the last page; publishing remains explicit.
- [x] Frontend build and 26 unit tests passed. One Playwright scenario passed with mocked API responses, covering cancellation, selection reset, CSRF propagation, busy controls, partial failure/retry and last-page recovery; desktop/mobile/landscape layout checks passed.
- [x] Linux/WSL fmt, workspace check, all-feature tests (62 passed, 10 ignored), Clippy, proxy-server build and real HTTP/recovery/password-reset/graceful-shutdown harness passed. The harness explicitly executed five ignored HTTP tests.

Evidence and limits: [mapping-bulk-delete.md](verification/mapping-bulk-delete.md). Browser bulk deletion was tested against mocked API responses; no live-backend browser bulk-delete test, deployment, Docker run or production benchmark is claimed. Separate PostgreSQL, Redis and scale tests were not rerun.

## Explicit revision cleanup

- [x] Add authenticated preview and explicit confirmation for old revision cleanup, defaulting to the latest 100 with a 1..10000 keep range and at most 500 deletions per batch. Only superseded revisions outside the newest N are eligible; runtime/database pointers and active/committed statuses remain protected.
- [x] Serialize cleanup with mutation/database locks, recompute and compare the complete preview, reject stale/tampered confirmation and commit deletion plus audit atomically. Preserve draft/source URLs, stable IDs, import retry results, version monotonicity and audit history. No automatic deletion or schema changes.
- [x] Add Revisions cleanup controls and handle invalid inputs, cancellation, busy state, stale conflict and repeated batches. Fix the preview-reset race found by the real browser test when active-revision data was stale.
- [x] Linux/WSL fmt/check/all-feature workspace tests (69 passed, 10 ignored), Clippy, server build and real HTTP/recovery/password-reset/graceful-shutdown checks passed. Disposable PostgreSQL cleanup/import-retry integration and HTTP checks passed. Frontend build, 26 unit tests, mocked browser scenario and real SQLite browser scenario passed.

Evidence and limits: [revision-cleanup.md](verification/revision-cleanup.md). The HTTP/PostgreSQL harnesses explicitly executed their ignored tests; separate Redis/scale tests were not rerun. Cleanup preserves protected revisions in addition to the newest N, requires a fresh preview per batch, and does not compact SQLite files or purge audit/import history. No deployment or production benchmark is claimed.
