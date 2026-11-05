# v1.1.0 release verification

Date: 2026-11-05.

## Scope

Stable release containing page-scoped bulk Mapping deletion and explicit old-revision cleanup. Functional evidence is recorded in [mapping-bulk-delete.md](mapping-bulk-delete.md) and [revision-cleanup.md](revision-cleanup.md), including real SQLite browser cleanup, PostgreSQL integration, protected snapshots, bounded batches, audit and import retry preservation.

Workspace and private frontend versions are aligned to 1.1.0. Cargo regenerated the lockfile offline; only eleven internal package versions changed from 1.0.0 to 1.1.0. Third-party versions, SQL migrations, existing data, runtime configuration and prior published tags remain unchanged. README/deployment instructions now target v1.1.0.

## Release validation

All commands below exited with code 0. Rust/service checks ran on Ubuntu 24.04 WSL with `CARGO_TARGET_DIR=/home/ubuntu/link-proxy-target`; frontend checks ran on Windows. Raw logs are local under `release-v1.1.0-*.log` and ignored by Git.

| Command | Result / log suffix |
| --- | --- |
| `cargo check --workspace --offline` | Regenerated lockfile; only eleven internal versions changed; `lock.log` |
| `cargo fmt --all -- --check` | Passed; `fmt.log` |
| `cargo check --workspace --locked` | Passed; `check.log` |
| `cargo test --workspace --all-features --locked` | 69 passed, 10 ignored; `test.log` |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed; `clippy.log` |
| `cargo build --locked -p proxy-server` | Passed; `build.log` |
| `python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Five real HTTP tests, recovery, password reset and graceful shutdown passed; `http.log` |
| `python3 scripts/postgres_test.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Disposable PostgreSQL including cleanup/import retry, HTTP and recovery passed; `postgres.log` |
| `python3 scripts/configuration_test.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Real Redis startup modes and invalid override rejection passed; `config.log` |
| `python3 -m unittest discover -s scripts -p test_package_release.py -v` | Three passed; `packaging.log` |
| `pnpm --dir apps/admin-web build` | Passed; `frontend-build.log` |
| `pnpm --dir apps/admin-web test` | 26 passed; `frontend-test.log` |

The HTTP and PostgreSQL harnesses explicitly executed their ignored tests. Browser feature checks are recorded in the linked feature evidence and were not repeated for the version-only preparation. Separate Redis advisory/import-scale tests and production benchmarks were not run. Docker runtime smoke, hosted deployment validation and asset verification remain pending until the release workflow runs.

## Publication status

The v1.1.0 tag, GitHub Release, hosted image smoke/publication and downloaded-asset verification are pending at preparation time. No deployment or production benchmark is claimed.

## Publication and actual assets verified

- Stable release, marked Latest: https://github.com/Dizzd/link-proxy/releases/tag/v1.1.0.
- Immutable tagged source: `739c25c504aa27bb0c2e8753b4753ce55f4fc4b1`.
- Successful hosted workflow: https://github.com/Dizzd/link-proxy/actions/runs/41981979444.
- All four jobs passed: Rust, frontend, deployment validation and publication. The publish job built linux/amd64 and ran the real PostgreSQL/proxy/Caddy/Redis smoke test before pushing the image and uploading all eight assets.
- Image tag: `ghcr.io/dizzd/link-proxy:v1.1.0`.
- Pinned image: `ghcr.io/dizzd/link-proxy@sha256:9f621e172af91f5fb1e29a0d2ed83aa5328756ebc7e38edb068f168dad4714af`.

Downloaded all eight actual assets through authenticated GitHub access. All seven SHA256SUMS entries matched; all six archive members were regular files with mode 0644 and matched standalone assets byte for byte. GitHub names the standalone environment asset `default.env.example`; the archive retains `.env.example`. `release.json` reports v1.1.0/linux/amd64 and matches the environment's pinned image and hosted publication logs. This supersedes the publication-pending status at preparation time.

Raw hosted logs, workflow/release snapshots and asset validation remain local under `release-v1.1.0-*`; downloads are under ignored `release-assets/v1.1.0`. Subsequent evidence documentation does not move the verified release tag. No production deployment or benchmark was performed.
