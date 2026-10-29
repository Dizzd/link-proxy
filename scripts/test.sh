#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
pnpm install --frozen-lockfile
pnpm --dir apps/admin-web build
pnpm --dir apps/admin-web test
cargo build --locked -p proxy-server
python3 scripts/e2e.py --binary "${CARGO_TARGET_DIR:-target}/debug/proxy-server"
python3 scripts/configuration_test.py --binary "${CARGO_TARGET_DIR:-target}/debug/proxy-server"
