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
