# v1.0.0 release verification

Date: 2026-11-02.

## Source preparation

Aligned the Rust workspace and private frontend package to 1.0.0. Cargo regenerated the lockfile offline; review confirmed that only the versions of eleven workspace packages changed. Dependencies, SQL migrations, runtime configuration, data and source URLs are unchanged. The README targets v1.0.0 and describes release deployment and operational limits.

## Executed local checks

Rust and service checks ran on Ubuntu 24.04 WSL; frontend checks ran on Windows. Each command below completed with exit code 0. Raw logs are local under `docs/verification/release-v1.0.0-*.log` and ignored by Git.

| Command | Result / log suffix |
|---|---|
| `cargo check --workspace --offline` | Lockfile regenerated; `lock.log` |
| `cargo fmt --all -- --check` | Passed; `fmt.log` |
| `cargo check --workspace --locked` | Passed; `check.log` |
| `cargo test --workspace --all-features --locked` | 62 passed, 10 ignored; `test.log` |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed; `clippy.log` |
| `cargo build --locked -p proxy-server` | Passed; `build.log` |
| `python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Five real HTTP tests, recovery, password reset and graceful shutdown passed; `http.log` |
| `python3 scripts/postgres_test.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Disposable PostgreSQL integration tests and HTTP/recovery harness passed; `postgres.log` |
| `python3 scripts/configuration_test.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Real Redis startup modes and invalid override rejection passed; `config.log` |
| `python3 -m unittest discover -s scripts -p test_package_release.py -v` | Passed; `packaging.log` |
| `pnpm --dir apps/admin-web build` | Passed; `frontend-build.log` |
| `pnpm --dir apps/admin-web test` | 22 passed; `frontend-test.log` |

The service harnesses explicitly execute ignored HTTP and PostgreSQL tests. Separate Redis advisory/scale tests, browser tests and benchmarks were not rerun locally. Docker execution, hosted release workflow, image publication and downloaded-asset verification are pending at source commit time. Local checks do not establish production capacity.

## Publication and downloaded assets verified

- Stable release, marked Latest: https://github.com/Dizzd/link-proxy/releases/tag/v1.0.0.
- Tagged source commit: `20df8f041022a4c2c65f84f9141c344a58c72d91`.
- Successful hosted workflow: https://github.com/Dizzd/link-proxy/actions/runs/41383412049.
- All four jobs passed: Rust, frontend, deployment validation and publication. The publication job built the linux/amd64 image and ran the real PostgreSQL/proxy/Caddy/Redis smoke test, including UI login, publication and a Caddy-to-Pingora redirect, before GHCR push and release asset upload.
- Image tag: `ghcr.io/dizzd/link-proxy:v1.0.0`.
- Immutable image: `ghcr.io/dizzd/link-proxy@sha256:60e6bbefabab2172cd876e42d6f94f66ae274ea94fa70d621f6cac666526166a`.

Downloaded all eight actual release assets through authenticated GitHub access. All seven SHA256SUMS entries matched. All six archive members were regular files with mode 0644 and matched their standalone assets byte for byte. GitHub names the standalone environment asset `default.env.example`; its archive name remains `.env.example`. `release.json` reports v1.0.0/linux/amd64 and its image digest matches `PROXY_IMAGE` and the hosted GHCR push logs. This supersedes the publication-pending status at source commit time.

Raw hosted logs and download-verification output remain local in `release-v1.0.0-hosted-full.log` and `release-v1.0.0-assets.log`; downloaded assets remain under ignored `release-assets/v1.0.0`. No production deployment or benchmark was performed. The published tag remains on the verified source commit; subsequent verification documentation does not retarget it.
