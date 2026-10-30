# Grouped navigation and monitoring verification

Executed on 2026-10-30. Rust commands ran under Ubuntu-24.04 WSL with CARGO_TARGET_DIR=/home/ubuntu/link-proxy-target. Frontend and Chromium ran on Windows.

| Check | Result | Evidence |
| --- | --- | --- |
| cargo fmt --all -- --check | Passed (no formatting differences) | release-monitoring-fmt.log |
| cargo check --workspace | Passed | release-monitoring-check.log |
| cargo test --workspace --all-features | 59 passed; 10 ignored | release-monitoring-test.log |
| cargo clippy --workspace --all-targets --all-features -- -D warnings | Passed | release-monitoring-clippy.log |
| cargo build -p proxy-server | Passed | release-monitoring-build.log |
| python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server | Five real Pingora HTTP tests, process recovery and password reset passed | release-monitoring-http.log |
| pnpm --dir apps/admin-web test | 22 passed | release-monitoring-frontend-test.log |
| pnpm --dir apps/admin-web build | Passed; existing dependency module-directive warnings | release-monitoring-frontend-build.log |
| Playwright against an isolated real backend | All 3 workflows passed | release-monitoring-chromium.log |

The browser harness used scripts/browser_test.py --external and Windows Node; direct WSL invocation of powershell.exe failed with Exec format error, so the external-runner mode completed the browser verification. The final run uses a fresh isolated database and process. Screenshots were visually inspected: monitoring-desktop.png and monitoring-mobile.png. Existing administration desktop/mobile screenshots were refreshed by that workflow.

The new monitoring workflow supplies test-only authenticated session, cache and metric responses. It verifies keyboard collapse/expand, charts from successive scrapes, exact latency values, pause/resume, refresh failure and recovery, and no horizontal page overflow at 375px and 850px. The existing administration workflow exercises real password sessions, publication, routing, import/export and logout against the isolated backend.

History is an in-page buffer of at most 60 successful scrapes, not durable historical monitoring. Chart values use counter/histogram deltas; cumulative summary cards are explicitly labelled. Missing metrics, no completed requests, resets and long gaps do not produce invented zero-latency or negative-rate measurements. Parser and calculation edge cases have five focused tests.

The remaining ignored external/scale tests, Docker builds, deployment and production benchmarks were not run. Existing SQL migrations, source URLs, operator configuration and published releases were not modified.
