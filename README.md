# dynamic-url-proxy

Dynamic URL mapping and conditional routing with a Pingora data plane, Axum management API and React administration dashboard. A QR identifier is an ordinary Mapping key; all matching, conditions and destination actions use the same domain.

## v1.0.0

Version 1.0.0 brings together conditional URL routing, authenticated administration, revision-based configuration and staged bulk import/export. Supported deployment choices are SQLite or PostgreSQL, with optional Redis synchronization and Caddy ingress.

The application image supports **linux/amd64**. The [v1.0.0 stable release](https://github.com/Dizzd/link-proxy/releases/tag/v1.0.0) is published. Its [release workflow](https://github.com/Dizzd/link-proxy/actions/runs/41383412049) passed Rust, frontend and deployment checks, image build, full-stack container smoke, GHCR push and upload of all eight assets. The downloaded bundle matched all seven checksums and six archive members. See [v1.0.0 verification evidence](docs/verification/release-v1.0.0.md).

CSV/XLSX bulk imports generate UUIDs when `external_id` is empty or omitted. Explicit IDs and IDs embedded in `mapping_json` are preserved. Generated IDs remain stable through preview and idempotent commit retries. Updating an existing `mapping_key` requires its correct stable ID. Each flat import row must provide its own `source_host`; blank cells do not inherit previous rows. Runtime deployments use one `config.toml`, with `CACHE_MODE` selecting `memory` or `memory_redis` at startup.

**Upgrading from v0.1.0:** preserve your effective TOML content as `config.toml`, including allowlists/authentication and Redis tuning. Replace the old file-selection variable with `CACHE_MODE` and use the new release image and Compose together. Retain the original database URL, Compose project name and volumes. The v0.1.0 image ignores `CACHE_MODE`; its published artifacts remain unchanged. See the [bundle README](deploy/README.md) for migration instructions.

Historical evidence is available for the [v0.3.1 pre-release](https://github.com/Dizzd/link-proxy/releases/tag/v0.3.1), its [successful workflow](https://github.com/Dizzd/link-proxy/actions/runs/41032028403) and [download verification](docs/verification/release-v0.3.1.md). Those results describe v0.3.1, not v1.0.0.

New release bundles contain dynamic-url-proxy-deploy.tar.gz, a single docker-compose.yml, .env.example, Caddyfile, config.toml, release metadata and SHA256SUMS. The bundle pins the exact GHCR image digest; no source build, Rust or Node is required on the deployment server.

## Capabilities

| Area | Implemented behavior |
|---|---|
| Routing | Host/method matching; indexed exact, template and prefix paths; deterministic priorities |
| Actions | Streaming reverse proxy, redirects, captured path rewriting and query policies |
| Conditions | Nested All/Any/Not, timezone-aware time windows, date ranges, device and query matching |
| Configuration | Validated drafts, explicit publication, immutable ArcSwap snapshots, revision history and rollback |
| Persistence | SQLite or PostgreSQL, embedded SQLx migrations and startup revision recovery |
| Import/export | CSV, XLSX and JSON; bounded dry-run/preview/staging; explicit idempotent batch commit |
| Administration | Username/password login, Argon2id hashing, HttpOnly sessions, CSRF protection and audit events |
| Operations | Caddy ingress, optional Redis synchronization, structured tracing and Prometheus metrics |

Normal routing requests read only the immutable in-memory snapshot; they never query SQL or Redis. Draft changes take effect after publication, and import commit publishes one validated batch.

## Deploy a release image

The Container release GitHub Action runs when a GitHub Release is published. It gates on Rust, frontend and deployment checks, builds the linux/amd64 image, smoke-tests PostgreSQL + proxy + Caddy + Redis, pushes the tested image to ghcr.io/dizzd/link-proxy:RELEASE_TAG and uploads an immutable-digest deployment bundle to that release. Release credentials use GITHUB_TOKEN; administrator/database passwords are set only on your server.

Download the runtime bundle and SHA256SUMS from [v1.0.0 Assets](https://github.com/Dizzd/link-proxy/releases/tag/v1.0.0) after the Container release workflow succeeds. For a private repository, use an authenticated browser or GitHub CLI to download the files. You can download on your workstation and upload the archive/checksums to the server; the server itself needs only Docker Engine and Docker Compose >= 2.20.0.

Use the archive to preserve all file names and relative mounts. GitHub lists the standalone environment asset as default.env.example; inside the archive it is .env.example as used below.

For a fresh deployment directory, the GitHub CLI option is:

```bash
mkdir -p ~/link-proxy
cd ~/link-proxy
# GitHub CLI is only needed on the machine downloading the private assets.
gh auth login
gh release download v1.0.0 --repo Dizzd/link-proxy \
  --pattern dynamic-url-proxy-deploy.tar.gz --pattern SHA256SUMS
tar -xzf dynamic-url-proxy-deploy.tar.gz
sha256sum -c SHA256SUMS
cp .env.example .env
chmod 600 .env
nano .env
# Set ADMIN_PASSWORD, POSTGRES_PASSWORD and PUBLIC_HOST.
docker compose config --quiet
# Private GHCR: enter a token with read:packages when prompted for a password.
docker login ghcr.io --username YOUR_GITHUB_USERNAME
docker compose pull
docker compose up -d --wait --wait-timeout 180
docker compose ps
```

The image targets **linux/amd64**. The package owner can make GHCR public for anonymous image pulls; publishing a release does not change visibility. Private packages require a user/token authorized to read that package; registry credentials are separate from the application's administrator account. Do not copy GitHub tokens into .env.

Select the `v1.0.0` tag explicitly so the downloaded bundle matches these instructions. Pre-releases are not resolved by GitHub's releases/latest download URL. For public downloads, the versioned archive URL is https://github.com/Dizzd/link-proxy/releases/download/v1.0.0/dynamic-url-proxy-deploy.tar.gz.

Set these environment values before startup for v1.0.0 (use the bundled README for older releases):

| Variable | Purpose |
|---|---|
| ADMIN_USERNAME / ADMIN_PASSWORD | First administrator; password must contain at least 12 bytes |
| POSTGRES_PASSWORD | Database password; use openssl rand -hex 32 to avoid URL escaping |
| PUBLIC_HOST | :80 for initial HTTP/IP access, or a DNS hostname such as links.example.com for HTTPS |
| COMPOSE_PROJECT_NAME | Stable installation name; keep it unchanged on upgrades |
| COMPOSE_PROFILES | postgres,caddy by default; add redis if required |
| CACHE_MODE | memory by default; memory_redis enables Redis using the same config.toml |

DATABASE_URL in the example derives from the PostgreSQL settings. URL-encode non-hex passwords explicitly. Application credentials are initialized only when the database has no administrator; restarting with another password does not reset an existing account.

One docker-compose.yml defines PostgreSQL, proxy, Caddy and Redis. The example COMPOSE_PROFILES=postgres,caddy starts the first three. PUBLIC_HOST=:80 serves HTTP/IP traffic; set your real DNS hostname for Caddy automatic HTTPS and open ports 80/443. Administration and direct proxy access remain on host loopback. Access Admin using ssh -N -L 19000:127.0.0.1:9000 USER@SERVER_IP, then open http://127.0.0.1:19000.

Enable Redis with COMPOSE_PROFILES=postgres,caddy,redis and CACHE_MODE=memory_redis. For SQLite, remove postgres from the profiles and set DATABASE_URL=sqlite:///data/proxy.db. Remove caddy if using an existing external ingress. No overlay files are required.

For source development, the root docker-compose.yml contains the same services but builds the application locally: copy .env.example to .env, edit it, then run docker compose up --build -d. Dockerfile, migrations and the built frontend are included in the published image; release hosts need no Rust/Node toolchain.

For upgrades, extract the new bundle into a separate directory and compare configuration. Keep the original COMPOSE_PROJECT_NAME, .env, config.toml, Caddyfile and database/certificate volumes; update only PROXY_IMAGE to the new release digest and reviewed deployment files. Then run docker compose pull and docker compose up -d --wait --wait-timeout 180. Do not overwrite the existing .env with the new example. Never use down -v on retained data. Existing SQLite data is not copied to PostgreSQL automatically. See [deployment](docs/DEPLOYMENT.md) and the bundle's README for backups, upgrades and options.

## Source development requirements

Linux (or WSL2), Rust stable, CMake, C/C++ toolchain, OpenSSL development headers, pkg-config; Node 22.12+ and pnpm 10.28.2 for the UI. SQLite is embedded; PostgreSQL is also supported. Redis is optional. Pingora 0.9 requires the workspace's Tokio **1.53.2 pin** because Tokio 1.54 rejects Pingora's zero-capacity transport buffers ([upstream issue](https://github.com/cloudflare/pingora/issues/1059)). Do not remove the pin without running live HTTP tests.

## Run from source

```bash
sudo apt-get install build-essential cmake pkg-config libssl-dev
export ADMIN_USERNAME=admin
read -rsp "Initial admin password (at least 12 bytes): " ADMIN_PASSWORD; echo
export ADMIN_PASSWORD
test -e config.toml || cp config/config.example.toml config.toml
pnpm install --frozen-lockfile
pnpm --dir apps/admin-web build
cargo run --locked -p proxy-server
```

Proxy: `http://localhost:8080`. Administration and built React UI: `http://127.0.0.1:9000`. Sign in with ADMIN_USERNAME and ADMIN_PASSWORD. The first startup stores an Argon2id password hash in the selected database; subsequent startups keep existing credentials. Sessions use HttpOnly, SameSite=Strict cookies, survive reload, and are revoked on logout or password change. Settings provides password changes and database status. Management APIs and metrics require authentication; minimal /health/live and /health/ready checks are public. For UI development use `pnpm --dir apps/admin-web dev` at port 5173, which forwards API requests to port 9000.

The default upstream CIDR allowlist is empty, denying every outbound proxy connection. Before using reverse proxy actions, edit `[security].upstream_cidr_allowlist` with narrowly scoped approved networks. HTTPS verification cannot be disabled. SQLite's directory is created at startup. No sample routes are activated automatically.

If config.toml already exists, edit that file instead of replacing it with the example.

## Database selection and password recovery

SQLite remains the default. To use an existing PostgreSQL server, create an empty application database and set the connection URL before startup:

```bash
export DATABASE_URL='postgres://proxy:URL_ENCODED_PASSWORD@127.0.0.1:5432/proxy'
cargo run --locked -p proxy-server
```

DATABASE_URL overrides database_url in TOML. PostgreSQL migrations run automatically. Use sslmode=verify-full and a trusted certificate for remote PostgreSQL. This selects a database; it does not copy existing SQLite data. Keep the SQLite file and use mapping export/import for a reviewed transfer (revision history and users are not transferred). Both backends keep normal proxy requests entirely in memory.

To reset a forgotten password, stop the server, set ADMIN_USERNAME to an existing user and ADMIN_PASSWORD to the new password, then run:

```bash
cargo run --locked -p proxy-server -- --reset-admin-password
```

The reset revokes every session. Changing environment variables alone never resets a stored password. After initial bootstrap you may remove ADMIN_PASSWORD from the service environment. Enable auth.secure_cookies behind HTTPS. Use one active control-plane process per database; imports recover interrupted jobs on startup. PostgreSQL publication uses row locks and optimistic revision checks to reject stale writers, but coordinated multi-writer import ownership is not implemented.

## First mapping

The UI handles login and CSRF automatically. For curl, first log in (requires jq):

```bash
session=$(curl --fail-with-body -sS -c /tmp/proxy-admin.cookies \
 -H 'Content-Type: application/json' http://127.0.0.1:9000/api/v1/auth/login \
 --data "$(jq -n --arg username "$ADMIN_USERNAME" --arg password "$ADMIN_PASSWORD" '{username:$username,password:$password}')")
csrf=$(printf '%s' "$session" | jq -r .csrf_token)
curl -sS http://127.0.0.1:9000/api/v1/mappings \
 -b /tmp/proxy-admin.cookies -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
 --data '{"mapping_key":"QR001","source_matcher":{"hosts":["abc.example.com"],"path_type":"exact","path_value":"/QR001","methods":["GET"]},"default_action":{"type":"redirect","status":302,"target_url":"https://example.org/"}}'
curl -sS http://127.0.0.1:9000/api/v1/config/publish \
 -b /tmp/proxy-admin.cookies -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
 --data '{"expected_revision":0}'
curl -i http://127.0.0.1:8080/QR001 -H 'Host: abc.example.com'
```

Use the current revision from `GET /api/v1/config/active` for subsequent publications. CRUD saves the draft; publish activates one compiled configuration. The UI's Revisions page provides the same operation. Import preview never changes the draft or the active snapshot; explicit import commit publishes exactly one revision.

## Workspace

- `apps/proxy-server`: Linux binary, private admin HTTP service, static UI, Pingora, background reconciliation.
- `apps/admin-web`: React, TypeScript, Vite, TanStack Query, React Router; ten administration pages.
- `crates/proxy-domain`: framework-independent entities and condition AST.
- `crates/proxy-routing`: indexed compiler, matching, condition evaluation, URL rewriting, simulator.
- `crates/proxy-pingora`: streaming data plane, IP-validated peer selection, forwarding policy.
- `crates/proxy-admin-api`: authenticated REST handlers.
- `crates/proxy-application`: aggregate edits, publication, import validation and atomic commit.
- `crates/proxy-storage`: SQLx SQLite/PostgreSQL persistence and repository boundary.
- `crates/proxy-cache`: ArcSwap configuration support, optional Redis envelopes and notifications.
- `crates/proxy-import`: bounded CSV/XLSX/JSON parsers and exports.
- `crates/proxy-observability`: structured logs and Prometheus metrics.
- `crates/proxy-e2e`: actual HTTP tests, run with the isolated service harness.
- Root Cargo.toml is a virtual workspace; Rust integration tests live with their owning crates. Root tests/ contains browser tests and shared import fixtures.

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
pnpm --dir apps/admin-web build
pnpm --dir apps/admin-web test
cargo build --locked -p proxy-server
python3 scripts/e2e.py
# Requires PostgreSQL server tools; creates a disposable cluster:
python3 scripts/postgres_test.py
# Requires redis-server; exercises CACHE_MODE using the same TOML file:
python3 scripts/configuration_test.py
bash scripts/benchmark.sh
```

Live HTTP tests are ignored in ordinary `cargo test` because they require real listeners; `scripts/e2e.py` starts an isolated Pingora process and echo backend and explicitly executes them. Scale imports and real Redis tests are also opt-in; see [Test plan](docs/TEST_PLAN.md). Results and actual limitations are recorded in [implementation progress](docs/IMPLEMENTATION_PLAN.md) and [benchmarks](docs/BENCHMARK.md).

## Operational limits

- Run one active control-plane process per database; import worker ownership and login rate limits are process-local.
- Changing DATABASE_URL selects another database; it does not copy users, mappings or revision history automatically.
- Administrator bootstrap/password recovery is implemented; multiuser/RBAC management is outside this release.
- Reverse proxy denies outbound connections until an explicit upstream CIDR allowlist is configured. No sample route is activated automatically.
- Configuration rollback restores a new revision; image rollback also requires database schema compatibility. Back up before upgrades.
- Historical benchmark measurements and their environments are documented separately; do not interpret them as production capacity guarantees.

## Examples and documents

Real import samples: `examples/import/basic.csv`, `examples/import/advanced.json`, `examples/import/advanced.xlsx`. Download live templates from the import UI or `/api/v1/imports/templates?format=xlsx`. Full conditional example: `tests/fixtures/conditional_mappings.json`.

Read [architecture](docs/ARCHITECTURE.md), [API contract](docs/API_CONTRACT.md), [routing semantics](docs/ROUTING_SPEC.md), [security](docs/SECURITY.md), [import/export](docs/BULK_IMPORT_EXPORT.md), and [configuration lifecycle](docs/CONFIGURATION_LIFECYCLE.md) before production deployment. Use one active control-plane process per database. PostgreSQL and SQLite use separate embedded migration directories; both support authenticated administration, staged imports and configuration recovery.

## Prepare and publish v1.0.0

Complete these steps in order:

1. Align `[workspace.package].version` in `Cargo.toml` and the private frontend version in `apps/admin-web/package.json` to `1.0.0`. Regenerate `Cargo.lock` with Cargo and review the workspace package version changes; keep dependency versions unchanged unless a separate change requires them.
2. Run the checks in [Verify](#verify) on Linux/WSL, including the real HTTP, recovery and shutdown harness. Execute any relevant opt-in tests explicitly. Record actual results and remaining limitations under `docs/verification` and in `docs/IMPLEMENTATION_PLAN.md`.
3. Review and commit the release changes, then create and push the `v1.0.0` tag from that commit. Create a GitHub Release for that exact tag; leave the pre-release option unchecked for the stable release.
4. Publishing the GitHub Release triggers the Container release workflow. Wait for Rust, frontend, deployment validation, image smoke testing, GHCR publication and asset upload to succeed. A tag push alone does not trigger this workflow.
5. Download the generated assets, check `SHA256SUMS` against the extracted files and archive, and confirm `release.json` and `PROXY_IMAGE` identify the same immutable image digest. Record the workflow URL, source commit and verified digest before announcing deployment availability.

The workflow takes the image tag and bundle version from the GitHub Release tag; it does not update Cargo or frontend package versions. Historical verification and benchmark results do not establish v1.0.0 acceptance or production capacity.

## Release workflow recovery

Maintainers can retry an existing release from its exact tag:

```bash
gh workflow run release.yml --repo Dizzd/link-proxy \
  --ref v1.0.0 --field release_tag=v1.0.0
```

The workflow requires its Git ref and release_tag to match, so the validation jobs and image build use the same source commit. It refuses branch-based builds for a release tag. Debian's built-in proxy system user is preserved; the application image runs under its own urlproxy account (UID 10001).
