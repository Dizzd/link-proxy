# Mapping bulk deletion verification

Date: 2026-11-05.

## Behavior

Mappings supports individual selection, mixed-state select-all for the current page (at most 25 rows), clear selection and a confirmation containing the selected count. Filter or page changes clear selection. Deletion reuses the existing authenticated, CSRF-protected DELETE API sequentially and stops on the first failure. The page reports completed deletions, refreshes the list and retains undeleted visible selections for review/retry. Filters, pagination and deletion controls are disabled during deletion. Deleting the final row on the last page returns to a valid page. Publishing remains explicit.

## Executed frontend checks

All commands below exited with code 0 on Windows:

| Command | Result | Local log |
| --- | --- | --- |
| `pnpm --dir apps/admin-web build` | TypeScript and Vite build passed | `mapping-bulk-delete-frontend-build.log` |
| `pnpm --dir apps/admin-web test` | 26 tests passed, including four deletion tests | `mapping-bulk-delete-frontend-test.log` |
| `pnpm exec playwright test tests/browser/mapping-bulk-delete.spec.ts` | One browser scenario passed | `mapping-bulk-delete-browser.log` |

The browser scenario used `PLAYWRIGHT_BASE_URL=http://127.0.0.1:5173` and a local Vite server with mocked session, cache status and mapping API responses. It verified select-all/mixed state, clear selection, filter/page reset, canceled confirmation without requests, CSRF header propagation, disabled controls while deleting, partial failure counts and retained selection, retry and last-page recovery. Layout checks passed at 1440×1000, 375×812 and 812×375 with reduced motion. Screenshots at each width remain local under `mapping-bulk-delete-<width>.png`; 375px and landscape screenshots were visually inspected. This browser test does not establish real backend bulk-delete integration.

## Linux checks

Checks ran in Ubuntu 24.04 WSL with `CARGO_TARGET_DIR=/home/ubuntu/link-proxy-target`:

| Command | Result | Local log |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Passed | `mapping-bulk-delete-fmt.log` |
| `cargo check --workspace --locked` | Passed | `mapping-bulk-delete-check.log` |
| `cargo test --workspace --all-features --locked` | 62 passed, 10 ignored | `mapping-bulk-delete-test.log` |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed | `mapping-bulk-delete-clippy.log` |
| `cargo build --locked -p proxy-server` | Passed | `mapping-bulk-delete-build.log` |
| `python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server` | Five real HTTP tests plus process recovery, password reset CLI and graceful shutdown passed | `mapping-bulk-delete-http.log` |

Each command exited with code 0. The HTTP harness explicitly executed its five ignored HTTP tests. PostgreSQL, Redis and import scale tests were not separately run for this frontend change. `git diff --check` also passed.

## Limits

Deletion uses separate requests and can partially succeed; it is not an atomic batch operation. Selection covers only the current page. No deployment, Docker runtime verification or production benchmark is claimed. Existing unrelated ignored external/scale tests are not implicitly counted as executed.
