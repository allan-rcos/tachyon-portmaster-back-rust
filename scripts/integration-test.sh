#!/usr/bin/env bash
#
# Runs the Go end-to-end integration suite. testcontainers-go builds the API
# image, starts the tmpfs MariaDB and the API pool, and tears everything down —
# no docker-compose file is involved.
#
# Env:
#   INTEGRATION_POOL_SIZE   number of {API + database} environments (default GOMAXPROCS)
#
# Usage: scripts/integration-test.sh [additional `go test` flags]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT/tests/integration"
exec go test ./... -count=1 -timeout 20m "$@"
