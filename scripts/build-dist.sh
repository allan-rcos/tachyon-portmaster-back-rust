#!/usr/bin/env bash
#
# Builds the deployable artifacts into dist/.
#
# Two tarballs, each with a .sha256 beside it:
#
#   portmaster-api-<version>-linux-x86_64.tar.zst   the static binary
#   portmaster-migrations-<version>.tar.zst         db/migrations + db/seeds
#
# **The migrations do not travel with the API.** The machine running the API is
# the one least entitled to alter the schema, and a deployment that *can*
# migrate is a deployment that can migrate by accident. Splitting the packages
# makes applying a migration a separate, deliberate act by someone who can see
# the database. See docs/adr/0008-minified-tarball-as-the-release-artifact.md.
#
# The binary is built for musl and linked statically, so the tarball unpacks
# onto a bare server with nothing installed — no runtime, no shared library, no
# package manager. That is the whole reason a tarball still makes sense next to
# the container image: an API process that *is* the web server has nothing to
# gain from a container runtime underneath it.
#
# Usage: scripts/build-dist.sh [version]
#
# With no argument the version comes from the workspace manifest, which is the
# same field the release workflow reads to decide whether to publish.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET=x86_64-unknown-linux-musl
DIST="$ROOT/dist"

version="${1:-}"
if [ -z "$version" ]; then
    version="$(cargo metadata --format-version 1 --no-deps \
        | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)"
fi
[ -n "$version" ] || { echo "could not determine the version" >&2; exit 1; }

echo "building portmaster $version for $TARGET"

# The target is added rather than assumed: on a machine that only ever built for
# the host, `cargo build --target` fails with an error about a missing std that
# does not say "run rustup target add".
rustup target add "$TARGET" >/dev/null

# --locked so the artifact is built from the exact versions that were tested. A
# release that silently picked up a new patch version of a dependency is a
# release nobody verified.
cargo build --release --locked --target "$TARGET" --bin portmaster-api-http

rm -rf "$DIST"
mkdir -p "$DIST/.stage/api" "$DIST/.stage/migrations"

install -m 0755 "target/$TARGET/release/portmaster-api-http" "$DIST/.stage/api/"
# Stripped: debug symbols are most of the binary and nothing on the server reads
# them. The unstripped copy stays in target/ for whoever needs to resolve a
# trace, and the .sha256 says which build it belongs to.
strip "$DIST/.stage/api/portmaster-api-http"

cp -r db/migrations db/seeds "$DIST/.stage/migrations/"

api="portmaster-api-${version}-linux-x86_64.tar.zst"
migrations="portmaster-migrations-${version}.tar.zst"

tar -C "$DIST/.stage/api" -c . | zstd -19 -q -o "$DIST/$api"
tar -C "$DIST/.stage/migrations" -c . | zstd -19 -q -o "$DIST/$migrations"

rm -rf "$DIST/.stage"

# Relative paths inside the checksum file, so `sha256sum -c` works from dist/
# regardless of where it was built.
( cd "$DIST" && sha256sum "$api" > "$api.sha256" && sha256sum "$migrations" > "$migrations.sha256" )

echo
ls -lh "$DIST"
