# Test Plan

## Required Linux verification
```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
pnpm --dir apps/admin-web build
pnpm --dir apps/admin-web test
cargo build --locked -p proxy-server
python3 scripts/e2e.py
```

Integration suites in proxy-routing, proxy-application and proxy-admin-api verify route indexes/precedence/methods, time/day/date boundaries, nested conditions, device heuristics, redirect policies, rewriting, config conflicts, ArcSwap isolation, API authentication/CRUD, staged imports, explicit commit/idempotency/cancellation, restart recovery and cache integrity/failure. Owning crates add 405 path detection, DST overnight regression and XLSX dependency/negative-priority round trips.

The five proxy-e2e integration binaries are ignored during ordinary cargo test. The isolated Python harness starts a real Pingora/Axum process, temporary SQLite database and real echo backend; it sets environment and explicitly runs all five with --ignored --test-threads=1. It verifies proxy streaming/rewrite/header forwarding, static/conditional redirects, two hostnames, denied metadata destination, runtime publication/rollback, all three import formats, idempotent commit, export, authentication and process restart. It also validates graceful shutdown. Simulator statistics are compared before/after without proxy traffic.

## Scale imports
```bash
cargo test -p proxy-application --test imports import_large_10000_and_50000_records -- --ignored --nocapture
```
This explicitly stages and activates 10,000 and 50,000 mappings, reporting wall-clock batch times. The ordinary suite includes a 1,000-record commit. Criterion tests lookup/compile at 1k/10k/100k and CSV parse at 1k/10k/50k; benchmark commands are in BENCHMARK.md.

## Redis
```bash
REDIS_TEST_URL=redis://127.0.0.1:6379/0 cargo test -p proxy-application --features redis-cache --test cache real_redis -- --ignored --nocapture
```
Ordinary tests validate memory-only independence, stale/tampered envelopes and unavailable-Redis local serving. The explicit test requires a real running Redis. A disposable harness is provided by scripts/redis_test.py when available. An ignored test is not considered executed unless its opt-in command succeeds.

## Frontend
Vitest covers filter encoding and condition defaults. Browser tests use Playwright against real private APIs where configured. `pnpm --dir apps/admin-web test` does not imply browser interaction coverage; see verification logs for actual commands. Check keyboard labels, password login, HttpOnly cookie restore, CSRF, logout and password change, loading/error states, create/edit/filter/pagination, nested rule editing, simulator, upload preview/commit, exports and revision restoration.

## Unverified acceptance gates
Docker must be built/run using a real engine. Public TLS, multi-hop redirect loop detection, production load/QPS and TLS certificate rejection need deployment-specific tests. Benchmarks are microbenchmarks and are not throughput or SLA claims. Current evidence belongs in IMPLEMENTATION_PLAN.md and docs/verification; never infer passed status from source presence.

## Reproduce the browser workflow on an isolated backend
Build the UI and backend first. Use an empty in-memory database for the browser test; the test creates the browser-mapping and example-001 fixtures.

Terminal 1 (Linux):
```sh
export ADMIN_API_TOKEN="$(openssl rand -hex 32)"
printf 'admin_listen="127.0.0.1:9000"\nproxy_listen="127.0.0.1:8080"\ndatabase_url="sqlite::memory:"\nadmin_static_dir="apps/admin-web/dist"\n' > /tmp/proxy-browser-test.toml
PROXY_CONFIG=/tmp/proxy-browser-test.toml cargo run -p proxy-server
```
Terminal 2: set the same ADMIN_API_TOKEN from the isolated server environment, then run `pnpm exec playwright install chromium` and `pnpm exec playwright test`. The test consumes the built UI served on private port 9000, not a mocked API or a Vite development server. Keep production listeners stopped or use an isolated environment for these test ports.

## PostgreSQL and session authentication

Run python3 scripts/postgres_test.py with PostgreSQL server tools installed. It creates an owned trust-authenticated cluster on a random loopback port, migrates fresh databases, tests revision concurrency/import/rollback/recovery and runs the same real Pingora HTTP suite. It never uses an existing application database. Normal cargo test skips the PostgreSQL integration test deliberately.

Authentication integration tests cover Argon2id storage, hashed session IDs, generic invalid credentials, CSRF and cross-origin rejection, logout, password change and revocation, expiration, disabled users, cookie Secure configuration, bounded malformed input, bootstrap preservation and persisted sessions. SQLite upgrade verification applies only original migrations first, persists mappings/revisions, then applies the additive auth migration and checks identities/URLs/history.

The real Chromium workflow now signs in using ADMIN_USERNAME/ADMIN_PASSWORD, keeps the session after reload, changes the password, rejects the old password and signs out. It also checks HttpOnly/SameSite cookie flags and database status. python3 scripts/browser_test.py starts an isolated backend and invokes Playwright; use --windows from WSL when Windows interop is available. If interop is unavailable, --external waits for scripts/browser_test.ps1 -SettingsPath PATH to write its result. Temporary test settings are deleted with the isolated fixture. Add --browser-external to postgres_test.py to exercise the browser on PostgreSQL as well.
