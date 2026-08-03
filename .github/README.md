# `.github/`

Two workflows: [`workflows/ci.yml`](workflows/ci.yml) checks what lands on main,
and [`workflows/release.yml`](workflows/release.yml) builds the production
artifact.

## Jobs

Two, independent and parallel — a clippy failure does not hide a test failure.

| Job | Runs | Typical |
|---|---|---|
| `rust` | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, release build | ~3–6 min |
| `go-integration` | `scripts/integration-test.sh` | 5–15 min |

### `rust`

Four commands, in the order that fails cheapest first: formatting, then lint,
then tests, then a release build.

**`--all-targets` on clippy.** Without it the lint stops at the library and the
binary, and a warning in test code only surfaces when someone runs the tests —
which is late, and in a place nobody is looking at lint output.

**The release build is not redundant.** It catches what a debug build does not:
an overflow that only panics with debug assertions on, a dependency that fails
to link optimised, a `--locked` that no longer resolves. It is also the profile
the deployed binary is built with, so it is the one worth proving compiles.

**The toolchain comes from `rust-toolchain.toml`.** `dtolnay/rust-toolchain@stable`
reads it, so the version CI uses and the version a developer gets locally cannot
drift apart.

### `go-integration`

The slow one, by a wide margin: it builds the API image and restarts a container
per test. Two things about it are deliberate.

**`timeout-minutes: 30`.** Without a ceiling, a hung container burns the default
six hours before GitHub gives up. Thirty minutes is comfortably above the real
runtime and fails fast when something wedges.

**The staleness gate.** Before running anything it installs `flatc`, regenerates
the Go bindings, and fails on a diff:

```yaml
- name: Verify generated Go bindings are up to date
  run: |
    scripts/generate-flatbuffers-go.sh
    git diff --exit-code tests/integration/internal/fbs \
      || (echo "::error::Go FlatBuffers bindings are stale — run scripts/generate-flatbuffers-go.sh" && exit 1)
```

The bindings are committed so the test runtime never needs `flatc`. This is what
stops that convenience from letting the contract silently drift between the API
and the tests. If it fires, run the script locally and commit the result.

`flatc` is a **test-side tool only**. The API generates its own wire types from
the same schemas at build time, in Rust, through `planus` — there is no `flatc`
in the Dockerfile and none in the image. Bumping the pinned version here changes
what the tests decode with, not what the API encodes with; the two agree because
both read `swagger/flatbuffers/schemas/`.

## Release

`workflows/release.yml`, one job. It runs `scripts/build-dist.sh` and nothing
else — anything CI does here, a developer can do with the same command.

**The version decides, not the trigger.** Every push to `main` builds — the
build has to be exercised on ordinary days, because release day is the worst one
to find out that assembling it is broken. Whether that build is *published* is
decided by `version` in the workspace `Cargo.toml`: a version with no
`v<version>` tag yet cuts the release and creates the tag on that commit; a
version already released stops after proving the build still works. Pushing a
tag by hand still publishes that tag.

The test is **"no such tag exists"**, not "`Cargo.toml` changed in this push".
On the ordinary path the two agree, and the first keeps being right across a
re-run, a batch of commits landing at once, or a rewritten history — and it is
what guarantees an existing release is never replaced. `concurrency` covers the
remaining hole, two pushes landing together and both finding the tag missing.

**`permissions: contents: write`.** `softprops/action-gh-release@v2` authenticates
with the automatic `GITHUB_TOKEN`, so **no repository secret is involved** — but
that token is read-only by default and the publish step 403s without this line.
A secret only enters the picture if the release is ever published to a different
repository than the one running the workflow.

**`musl-tools` is not optional.** The artifact is statically linked against musl
so it unpacks onto a bare server, and the dependencies carrying C — `ring`, from
the TLS stack — need a musl `cc` to build. Without the package the job fails
with a linker error that does not mention musl.

To cut a release: bump `version` in `[workspace.package]` and push to `main`.
The tag and the release are made for you; there is no secret to configure first.

## Submodules

Every job checks out with:

```yaml
- uses: actions/checkout@v4
  with:
    submodules: recursive
```

The FlatBuffers schemas live in the `swagger/` submodule, and `build.rs`
generates the wire types from them at compile time. Without the submodule the
build fails with the message `build.rs` writes itself — but only after the
checkout has already succeeded, which is why the flag is on every job rather
than remembered per job.

## Adding a job

Match the existing shape — checkout with submodules, set up the toolchain, then
one command. Keep jobs independent: a new one should not need another to have
passed first. If it can take an unbounded amount of time, give it a
`timeout-minutes`.

Anything a job runs should be runnable locally by the same command. If CI needs
a step a developer cannot reproduce, put it in `scripts/` first.
