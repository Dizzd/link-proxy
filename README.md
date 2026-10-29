# dynamic-url-proxy

Unified Dynamic URL Mapping & Conditional Routing Engine. Pingora streams proxy traffic; Axum manages authenticated drafts, immutable revisions, simulation, and staged imports. QR identifiers are ordinary mapping keys.

## Requirements

Linux (or WSL2), Rust stable, CMake, C/C++ toolchain, OpenSSL development headers, pkg-config; Node 22.12+ and pnpm 10.28.2 for the UI. SQLite is embedded; PostgreSQL is also supported. Redis is optional. Pingora 0.9 requires the workspace's Tokio **1.53.2 pin** because Tokio 1.54 rejects Pingora's zero-capacity transport buffers ([upstream issue](https://github.com/cloudflare/pingora/issues/1059)). Do not remove the pin without running live HTTP tests.

## Run locally

```bash
sudo apt-get install build-essential cmake pkg-config libssl-dev
export ADMIN_USERNAME=admin
read -rsp "Initial admin password (at least 12 bytes): " ADMIN_PASSWORD; echo
export ADMIN_PASSWORD
cp config/config.example.toml config.toml
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
bash scripts/benchmark.sh
```

Live HTTP tests are ignored in ordinary `cargo test` because they require real listeners; `scripts/e2e.py` starts an isolated Pingora process and echo backend and explicitly executes them. Scale imports and real Redis tests are also opt-in; see [Test plan](docs/TEST_PLAN.md). Results and actual limitations are recorded in [implementation progress](docs/IMPLEMENTATION_PLAN.md) and [benchmarks](docs/BENCHMARK.md).

## Deploy a published release (no source build on the server)

The Container release GitHub Action runs when a GitHub Release is published. It gates on Rust, frontend and deployment checks, builds the linux/amd64 image, smoke-tests PostgreSQL + proxy + Caddy + Redis, pushes the tested image to ghcr.io/dizzd/link-proxy:RELEASE_TAG and uploads an immutable-digest deployment bundle to that release. Release credentials use GITHUB_TOKEN; administrator/database passwords are set only on your server.

After the first release succeeds, download dynamic-url-proxy-deploy.tar.gz from its Assets:

```bash
mkdir -p ~/link-proxy
cd ~/link-proxy
curl -fLO https://github.com/Dizzd/link-proxy/releases/latest/download/dynamic-url-proxy-deploy.tar.gz
tar -xzf dynamic-url-proxy-deploy.tar.gz
cp .env.example .env
chmod 600 .env
nano .env
# Set ADMIN_PASSWORD, POSTGRES_PASSWORD and PUBLIC_HOST.
docker compose config --quiet
docker compose pull
docker compose up -d --wait --wait-timeout 180
docker compose ps
```

The server requires Docker Compose >= 2.20.0 and linux/amd64. For private repositories/packages, use authenticated Release downloads and docker login ghcr.io with read:packages access. The package owner can make GHCR public for anonymous pulls; publication does not change visibility automatically.

One docker-compose.yml defines PostgreSQL, proxy, Caddy and Redis. The example COMPOSE_PROFILES=postgres,caddy starts the first three. PUBLIC_HOST=:80 serves HTTP/IP traffic; set your real DNS hostname for Caddy automatic HTTPS and open ports 80/443. Administration and direct proxy access remain on host loopback. Access Admin using ssh -N -L 19000:127.0.0.1:9000 USER@SERVER_IP, then open http://127.0.0.1:19000.

Enable Redis with COMPOSE_PROFILES=postgres,caddy,redis and PROXY_CONFIG_FILE=config.redis.toml. For SQLite, remove postgres from the profiles and set DATABASE_URL=sqlite:///data/proxy.db. Remove caddy if using an existing external ingress. No overlay files are required.

For source development, the root docker-compose.yml contains the same services but builds the application locally: copy .env.example to .env, edit it, then run docker compose up --build -d. Dockerfile, migrations and the built frontend are included in the published image; release hosts need no Rust/Node toolchain.

Keep the original COMPOSE_PROJECT_NAME, .env, configuration and database/certificate volumes on upgrades; update only PROXY_IMAGE to the new release digest and reviewed deployment files. Never use down -v on retained data. Existing SQLite data is not copied to PostgreSQL automatically. See [deployment](docs/DEPLOYMENT.md) and the bundle's README for backups, upgrades and options.

## Examples and documents

Real import samples: `examples/import/basic.csv`, `examples/import/advanced.json`, `examples/import/advanced.xlsx`. Download live templates from the import UI or `/api/v1/imports/templates?format=xlsx`. Full conditional example: `tests/fixtures/conditional_mappings.json`.

Read [architecture](docs/ARCHITECTURE.md), [API contract](docs/API_CONTRACT.md), [routing semantics](docs/ROUTING_SPEC.md), [security](docs/SECURITY.md), [import/export](docs/BULK_IMPORT_EXPORT.md), and [configuration lifecycle](docs/CONFIGURATION_LIFECYCLE.md) before production deployment. Use one active control-plane process per database. PostgreSQL and SQLite use separate embedded migration directories; both support authenticated administration, staged imports and configuration recovery.
