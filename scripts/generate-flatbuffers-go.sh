#!/usr/bin/env bash
#
# Regenerates the Go FlatBuffers bindings used by the integration suite from the
# canonical schemas in the swagger submodule. The output is committed under
# tests/integration/internal/fbs, so the test runtime never needs flatc; CI only
# runs this to verify the committed bindings are current (git diff --exit-code).
#
# Usage: scripts/generate-flatbuffers-go.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/tests/integration/internal/fbs"
SCHEMAS="$ROOT/swagger/flatbuffers/schemas"

command -v flatc >/dev/null 2>&1 || { echo "flatc not found on PATH" >&2; exit 1; }

rm -f "$OUT"/*.go
mkdir -p "$OUT"

# --go-namespace fbs collapses every schema's namespace into a single flat Go
# package, so the suite imports one `fbs` package for all message types.
flatc --go --go-namespace fbs -o "$ROOT/tests/integration/internal" "$SCHEMAS"/*.fbs

# flatc does not rewrite cross-file enum/table references to the overridden
# namespace: it leaves dangling `API__Fbs__X "API/Fbs/X"` imports and prefixes,
# even though those types now live in the same `fbs` package. Strip both so the
# package is self-contained and compiles.
perl -i -ne 'print unless /^\s*API__Fbs__\w+ "API\/Fbs\/\w+"$/' "$OUT"/*.go
perl -i -pe 's/API__Fbs__\w+\.//g' "$OUT"/*.go
gofmt -w "$OUT" 2>/dev/null || true

echo "generated $(ls "$OUT"/*.go | wc -l | tr -d ' ') Go binding files in tests/integration/internal/fbs"
