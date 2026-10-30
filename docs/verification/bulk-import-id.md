# Optional mapping IDs in bulk imports

Date: 2026-10-30. Rust checks executed on WSL Ubuntu-24.04; frontend commands executed on Windows.

CSV/XLSX mapping rows now accept an absent or empty external_id. Flat rows receive UUIDs; mapping_json keeps its existing ID unless a nonempty external_id overrides it. Explicit IDs and existing key-conflict policies remain unchanged. Generated IDs are staged before explicit commit and retained on an idempotent commit retry. The regression suite also checks that a fresh upload cannot overwrite an existing mapping_key by omitting its ID.

Executed checks:

- cargo fmt --all -- --check: passed (bulk-import-id-fmt.log).
- cargo check --workspace: passed (bulk-import-id-check.log).
- cargo test --workspace --all-features: 62 passed, 0 failed, 10 ignored (bulk-import-id-test.log).
- cargo clippy --workspace --all-targets --all-features -- -D warnings: passed (bulk-import-id-clippy.log).
- cargo build -p proxy-server: passed (bulk-import-id-build.log).
- pnpm --dir apps/admin-web build: passed (bulk-import-id-web-build.log).
- pnpm --dir apps/admin-web test: 22 passed (bulk-import-id-web-test.log).
- python3 scripts/e2e.py --binary /home/ubuntu/link-proxy-target/debug/proxy-server: passed; explicitly executed the five ignored HTTP tests, process recovery, graceful shutdown and password reset (bulk-import-id-e2e.log).

The original Downloads/template.xlsx was parsed directly with the Rust importer through a temporary local diagnostic example, subsequently removed. It produced seven mappings with seven nonempty IDs and six invalid_mapping errors with description `invalid hostname`, matching the missing source_host cells in Mappings rows 3-8 (bulk-import-id-workbook.log). The original workbook was not modified and no uploaded data was committed to a running deployment.

No Docker runtime, deployment or production benchmark is claimed. Ignored external/scale tests outside the HTTP harness have not been executed for this fix.
