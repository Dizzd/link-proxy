# v1.0.0 README preparation verification

Date: 2026-11-02.

Scope: update the root README for the planned v1.0.0 release. Deployment commands and workflow recovery now target v1.0.0; v0.3.1 links remain historical evidence. The README describes release preparation, publication gates, operational limits and retained-data upgrades. No source code, dependencies, configuration, migrations or package versions were changed.

Executed against the current checkout (Rust workspace version 0.3.1), using Ubuntu 24.04 on WSL for Rust and the HTTP harness, and Windows for the frontend:

| Check | Result | Local raw log |
|---|---|---|
| `cargo fmt --all -- --check` | Exit 0 | `readme-v1.0.0-fmt.log` |
| `cargo check --workspace --locked` | Exit 0 | `readme-v1.0.0-check.log` |
| `cargo test --workspace --all-features --locked` | 62 passed, 10 ignored; exit 0 | `readme-v1.0.0-test.log` |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Exit 0 | `readme-v1.0.0-clippy.log` |
| `cargo build --locked -p proxy-server` | Exit 0 | `readme-v1.0.0-build.log` |
| `python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Five live HTTP tests passed; recovery, password reset and graceful shutdown checks passed; exit 0 | `readme-v1.0.0-http.log` |
| `pnpm --dir apps/admin-web build` | Exit 0 | `readme-v1.0.0-frontend-build.log` |
| `pnpm --dir apps/admin-web test` | 22 passed; exit 0 | `readme-v1.0.0-frontend-test.log` |

`git diff --check` passed, and all 12 local Markdown file links in README resolved. Release workflow instructions were compared with `.github/workflows/release.yml` and `scripts/package_release.py`; no external release state was queried.

The HTTP harness explicitly executed the five ignored HTTP tests. The remaining ignored Redis/scale tests, dedicated PostgreSQL and runtime cache configuration harnesses, browser tests, Docker runtime, publication and benchmarks were not executed for this documentation change. Raw logs are local and ignored by Git. These results do not verify a v1.0.0 tag or image.

Before tagging, package versions still need alignment: `Cargo.toml` is 0.3.1 and `apps/admin-web/package.json` is 0.1.0. Release publication and downloaded-asset verification remain pending.
