#!/usr/bin/env bash
set -euo pipefail
: "${ADMIN_USERNAME:?Set the initial admin username}"
: "${ADMIN_PASSWORD:?Set an initial password with at least 12 bytes}"
mkdir -p data
pnpm install --frozen-lockfile
pnpm --dir apps/admin-web build
exec cargo run -p proxy-server
