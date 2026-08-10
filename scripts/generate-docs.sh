#!/usr/bin/env bash
#
# Renders the doc comments in crates/ into browsable HTML under docs/rustdoc.
#
# The output is committed rather than gitignored, because GitHub Pages serves
# this repository's docs/ directory — so the rendered API reference is only
# published if it is in the tree. The build itself stays under target/, which is
# not.
#
# Unlike the PHP side, there is no tool to install and no version to pin:
# rustdoc ships with the toolchain that rust-toolchain.toml already fixes. That
# is the whole reason this script is fifty lines instead of a container
# fallback.
#
# --document-private-items is a default here rather than a flag a caller has to
# remember. Every implementation under `intern/` is private by design — the
# contracts are the published surface and the types behind them are not — and
# without the flag rustdoc renders the contracts and silently drops ~120
# implementations, along with every note written on them. Drop it for the
# contract-only view.
#
# Env:
#   RUSTDOCFLAGS   passed through; the CI sets `-D warnings`
#
# Usage: scripts/generate-docs.sh [additional `cargo doc` flags]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/docs/rustdoc"

cd "$ROOT"

# A broken `[`Item`]` link renders as plain text with no marker in the page, so
# it is invisible in the very artefact it damages. Failing here is the only way
# a reader ever finds out — and it is what the CI does too.
export RUSTDOCFLAGS="${RUSTDOCFLAGS:--D warnings}"

# --no-deps: the dependency docs are on docs.rs and rendering them locally turns
# a 12-second build into minutes of HTML nobody reads from here.
cargo doc --workspace --no-deps --document-private-items "$@"

# `cargo doc` writes to target/doc, which is gitignored. Publishing means
# copying, and the delete-first is deliberate: without it, a module that was
# renamed leaves its old page behind forever, reachable by anyone holding the
# link and describing code that no longer exists.
rm -rf "$OUT"
mkdir -p "$OUT"
cp -r target/doc/. "$OUT/"

# target/doc has no index.html of its own — `cargo doc --open` guesses a crate.
# Pages needs a real one, so the entry point is written here.
cat > "$OUT/index.html" <<'HTML'
<!doctype html>
<meta charset="utf-8">
<title>Portmaster — referência da API</title>
<meta http-equiv="refresh" content="0; url=portmaster_domain/index.html">
<p>Redirecionando para <a href="portmaster_domain/index.html">portmaster_domain</a>.
HTML

echo "docs/rustdoc pronto — abra docs/rustdoc/index.html"
