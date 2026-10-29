#!/usr/bin/env bash
set -euo pipefail
cargo bench -p proxy-routing --bench routing_bench
cargo bench -p proxy-import --bench import_bench
cargo run --release -p proxy-routing --example memory
