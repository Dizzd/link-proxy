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
