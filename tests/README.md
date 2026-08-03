# `tests/`

Two suites, and only one of them lives here.

```
tests/
└── integration/        Go — the API over real HTTP
```

The unit tests are not in this directory at all: in Rust they live next to the
code they cover, in a `#[cfg(test)] mod tests` at the foot of each module. That
is not a convention borrowed for its own sake — it is what lets a test reach a
`pub(crate)` type without the crate publishing it, which is exactly what testing
a table module or a repository requires.

| | `cargo test` | `integration/` |
|---|---|---|
| Tool | the built-in harness | `go test` + testcontainers |
| Lives in | `crates/*/src/**`, beside the code | here |
| Covers | rules and control flow | the API as a client sees it |
| Talks to | fakes and hand-written stubs | real HTTP, real MariaDB |
| Run | `cargo test --workspace` | `scripts/integration-test.sh` |
| Costs | ~4 s for all of them | ~20 s per leased environment |

**The dividing line:** if a behaviour is observable through a request and a
response, it is an integration test. If it is a rule or a branch, it is a unit
test. Testing a domain rule over HTTP is slow and indirect; testing a status
code with mocks proves only that the mocks agree with each other.

## Unit tests, by layer

### `domain` — the rules

Table modules are where validation lives, so they are tested directly, with no
doubles. One case per rule, plus the happy path, asserting **which fields** were
refused rather than only that something was:

```rust
let error = tm.create(String::new(), 1.0, RiskClass::Class3FlammableLiquids)
    .expect_err("um produto sem nome não deveria ser construído");
```

The domain accumulates every refused field instead of stopping at the first, so
a test that asserts only "it failed" would pass while the batch quietly shrank
to one.

### `app` — the transactional spine

Use cases are tested for control flow, not business rules — those belong to the
table module. The same three things for every write use case:

1. **Commit on the happy path.**
2. **Rollback on every failure**, and never also commit.
3. **The 403 guard** — a context without the permission is refused before any
   work happens.

There is a fourth, and it is the one that pays for itself: `crates/app/src/lib.rs`
holds two tests that **never run**. They take a generic `P: AppProvider` and push
a use-case future through `tokio::spawn`, which demands `Send + 'static`. If a
port ever holds something `!Send` across an `await`, those stop compiling — here,
instead of in every axum handler three layers away.

### `infra` and `api-http`

`infra` tests the pieces that have no database in them: the SQL builder's bind
ordering, the cursor's filter identity, the row readers that fail rather than
default. `api-http` tests the wire in both directions, the token, the cookies and
the error mapping.

Neither reaches a real MariaDB — that is what the integration suite is for, and
duplicating it here would buy a slower version of a test that already exists.

## `integration/`

Go, driving the real API over HTTP with real FlatBuffers payloads against a real
MariaDB. Tests are **stories** — one narrative per domain over a single leased
environment, with sub-tests that run in order and share state, because the rules
worth testing here are transitions.

Full detail, including the factory layout and the comment convention:
[`integration/README.md`](integration/README.md).

## Running

```bash
cargo test --workspace                         # unit
cargo test -p portmaster-domain                # one crate
cargo test product::                           # one module
scripts/integration-test.sh                    # integration
scripts/integration-test.sh -run TestYardStory # one story
```

CI runs them as separate jobs — see [`.github/README.md`](../.github/README.md).

## Adding tests for a feature

1. A table-module test per rule you added, in the module itself.
2. A use-case test for commit, rollback and the 403.
3. Factories in `integration/internal/factories/<feature>.go`.
4. Steps appended to the story that owns the resource.
