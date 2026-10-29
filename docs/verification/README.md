# Verification evidence - 2026-10-29

Raw command logs in this directory record executed checks, not a planned acceptance list. Linux: Ubuntu WSL2, Rust 1.98.1. Frontend: Windows Node 25.2.1, Vite 7.3.7. Chromium workflow uses the built UI and a real isolated Pingora/Axum process.

| Evidence | Result |
|---|---|
| cargo-fmt.log, cargo-check.log | Formatting and workspace check passed |
| cargo-test.log | 47 ordinary Rust tests passed; 9 service/scale tests explicitly opt-in |
| http-e2e.log | 5 live HTTP tests passed; restart retained committed revision 10 |
| redis-tests.log | 3 real Redis tests passed, including advisory notifications and shutdown/failure retention |
| scale-imports.log | Opt-in scale test passed for 10k and 50k mappings |
| cargo-clippy.log | All workspace targets/features passed with -D warnings |
| frontend-build.log | TypeScript and production Vite build passed |
| frontend-test.log | 7 Vitest tests passed |
| browser-e2e.log | 1 Chromium workflow passed; process exit code 0 |
| routing-benchmark.log, import-benchmark.log | Actual release Criterion measurements |
| memory-benchmark.log, benchmark-environment.log | Actual process VmRSS and measurement host |
| admin-desktop.png, admin-mobile.png | Inspected real UI screenshots |

The raw debug logs can mention local filesystem/compiler paths but contain no production API tokens or Redis credentials. Browser test configuration is isolated in memory. Docker is not marked verified. See ../IMPLEMENTATION_PLAN.md and ../BENCHMARK.md for limitations and measurement interpretation.

## Compatibility-directory cleanup verification

Root Cargo.toml is now a virtual workspace with eleven package members. All 47 ordinary Rust tests were retained after relocation into their owning crates. After cleanup, cargo fmt/check, the complete all-features test suite, clippy with -D warnings and proxy-server build passed. The actual HTTP/process-recovery harness passed all five tests; the updated Redis harness passed all three tests from proxy-application. The canonical frontend build and all seven Vitest tests passed. Raw cleanup-specific output is in cleanup-*.log.

Removed directories: root admin/, root src/, generated test-results/, scripts/__pycache__/ and the emptied root tests/common/. Browser tests and fixtures remain under tests/. Only the unused root migration copy was removed; the embedded SQLite migration checksum was unchanged. Docker references were updated, but a container build was not executed because Docker remains unavailable in this environment.

## PostgreSQL/password extension, 2026-10-29

New password-*.log and postgres-*.log evidence is separate from historical token-UI runs. Browser tests use generated credentials in an isolated temporary fixture, never a production account. Temporary settings are ignored by Git and removed when the fixture closes. Performance results were not remeasured for this storage/auth extension; prior benchmarks describe their original SQLite builds.

## Release workflow and unified Compose

release-package-tests.log records packaging/secret-exclusion/digest/checksum tests. release-compose-validation.log records real Docker Compose CLI validation of release and source files across SQLite-only, SQLite+Caddy, PostgreSQL+Caddy and PostgreSQL+Caddy+Redis. release-actionlint.log is empty on successful actionlint validation. release-rust-*.log and release-ui-build.log record regression checks. These do not claim Docker Engine execution, registry publication or a GitHub Actions run; the full-stack image smoke is implemented for the hosted runner.
