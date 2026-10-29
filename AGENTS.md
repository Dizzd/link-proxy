# Dynamic URL Mapping & Conditional Routing Engine

Use English for code and technical documentation. Preserve existing data and URLs.
Mapping owns all conditions and destination actions; never introduce a separate QR evaluator.
Compile immutable snapshots before publishing. Request processing must not query SQL or Redis.
The selected SQLite or PostgreSQL database is authoritative. Database commit precedes ArcSwap activation; startup reconciles committed revisions.
Use additive SQLx migrations, explicit import commit, bounded uploads and workers, and authenticated administration.
Run `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings` on Linux, and `pnpm --dir apps/admin-web build`.
Do not report acceptance criteria or benchmarks as passed without execution evidence.

## Workspace and maintenance
- Domain is framework independent; Mapping owns the entire rule/action AST.
- Dependency direction: domain → routing/storage/cache/import → application → admin API/Pingora → binary. Do not introduce request-time persistence calls.
- Root Cargo.toml is a virtual workspace. Production source lives in apps/ and crates/; Rust integration tests live with their owning crates.
- Build Pingora on Linux/WSL. Keep Tokio =1.53.2 while Pingora 0.9 uses zero-capacity BufStream; see upstream cloudflare/pingora#1059 and run the real HTTP harness before changing versions.
- Embedded migration sources are migrations/sqlite and migrations/postgres, selected by backend. Never edit an applied migration; add a new numeric SQL migration.
- One active control-plane process per database; import worker ownership is local. Compile the full candidate before SQL commit. Commit immutable revision and pointer before ArcSwap. Recover committed revisions on startup.
- Preserve stable IDs, existing data and source URLs. No automatic sample import or silent overwrite. Import requires bounded validation, preview and explicit idempotent commit.
- Administration defaults to loopback. Require username/password sessions with Argon2id, HttpOnly cookies and CSRF validation; optional API tokens require explicit enablement. Require bounded bodies/workers and explicit CIDR allowlists. Never log tokens or Redis credentials. HTTPS upstream verification is mandatory.
- UI code and build commands live in apps/admin-web. Never embed credentials in Vite env variables or localStorage.
- Run cargo fmt --all -- --check, cargo check --workspace, cargo test --workspace --all-features, cargo clippy --workspace --all-targets --all-features -- -D warnings, pnpm --dir apps/admin-web build and pnpm --dir apps/admin-web test.
- Build proxy-server and run python3 scripts/e2e.py for actual HTTP, process recovery and graceful-shutdown verification. Ignored tests require explicit execution; record evidence under docs/verification and update docs/IMPLEMENTATION_PLAN.md honestly.
- Docker and production benchmarks require a real runtime; never mark them passed from file inspection.
