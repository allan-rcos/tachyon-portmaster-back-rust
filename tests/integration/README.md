# Integration suite

End-to-end tests written in Go, driving the real API over HTTP with real
FlatBuffers payloads against a real MariaDB. Nothing here reaches inside the
API: if a behaviour cannot be observed through a request and a response, it
belongs in the Rust unit tests instead (`cargo test --workspace`, see
[`../../README.md`](../../README.md)).

Run it:

```bash
scripts/integration-test.sh                 # the whole suite
scripts/integration-test.sh -run TestYard   # one story
INTEGRATION_POOL_SIZE=2 scripts/integration-test.sh   # fewer parallel environments
```

Requires Docker and Go 1.25. `flatc` is **not** required — the bindings under
`internal/fbs` are committed.

## Layout

```
tests/integration/
├── main_test.go              TestMain, the shared pool, and the assertion helpers
├── session_story_test.go     bootstrap, login, refresh, logout, password change
├── administration_story_test.go   roles, users, permissions
├── yard_story_test.go        products, containers, manifests, metrics
└── internal/
    ├── harness/              testcontainers: the pool, MariaDB, migrations, credentials
    ├── client/               HTTP transport — cookie jar, verbs, session helpers
    ├── factories/            request payload builders, one file per feature
    └── fbs/                  GENERATED — do not edit
```

### `internal/fbs` is generated

`scripts/generate-flatbuffers-go.sh` regenerates it from the canonical schemas in
the `swagger/` submodule; the output is committed so the test runtime never needs
`flatc`. CI regenerates and runs `git diff --exit-code` against it, so a schema
change that is not accompanied by regenerated bindings fails the build.

### `internal/harness`

Builds the API image once, starts one tmpfs MariaDB, and provisions a pool of
`{API container + database}` environments — `INTEGRATION_POOL_SIZE`, defaulting
to `GOMAXPROCS`. A test takes one with `pool.Lease(t)` and gets it back clean;
it is returned to the pool automatically at the end of the test.

`Lease` costs roughly twenty seconds, because a reset is a schema drop, a
re-migrate **and** an API restart. The restart is not optional: the permission
catalogue and the `refresh-token` marker group live in the API process and are
filled exactly once, at boot. Without the restart every test after the first
would run against a server holding a catalogue built for a database that no
longer exists — and, worse, sessions marked in a registry the new schema knows
nothing about. That price is what shapes the whole suite — see below.

> This used to be phrased in terms of `ENGINE=MEMORY` tables, which the schema
> drop took with it. The registries moved into the process
> ([ADR 0009](../../docs/adr/0009-runtime-registries-in-process.md)); the reason
> for the restart survived the move intact.

### `internal/client`

The HTTP layer: `Get`/`Post`/`Put`/`Delete`, a cookie jar per client, and
`Cookie`/`SetCookie` for the tests that need to tamper with a session. `Setup`
and `LoginAs` send a request and assert the happy path; when a test wants to
drive the response itself, it builds the body with a factory and posts it.

A second `client.New(env.BaseURL)` is how a test gets an *anonymous* or *other*
caller against the same environment — that is the whole mechanism for testing
"someone else's token does not work here".

### `internal/factories`

One file per feature: `product.go`, `container.go`, `manifest.go`, `role.go`,
`user.go`, `account.go`, `auth.go`. A factory for a route under `/products` goes
in `product.go` — including the deliberately invalid payloads for that feature,
which sit next to the valid ones so that adding a rule and adding its
counter-example land in the same file.

Factories returning a struct (`Product`, `Container`, `Role`, `User`) carry both
the encoded `Bytes` and the values that went into them, so a test can create a
resource and assert the server echoed those values back. The rest return a bare
`[]byte`.

## Stories, not tests per endpoint

Each `*_story_test.go` file is one narrative over a single leased environment,
and its sub-tests run **in order, not in parallel** — each depends on the state
the previous one left.

That is a deliberate trade against the twenty-second lease. Spending it per
assertion would buy isolation nobody needs here, and it would lose the thing
these tests are actually good at: the *order*. Session bugs live in sequences —
a token that still works after logout, a rotation that outlives its predecessor —
and only a story can catch them.

Stories run in parallel with **each other** (`t.Parallel()` at the top), each on
its own environment.

**Adding a step to an existing story** is the default: a new endpoint on an
existing resource is one more `t.Run` in the story that already owns that
resource. **Opening a new story** is for a genuinely separate narrative with its
own bootstrap — expect it to cost another environment's worth of wall time.

## Comment convention

The inconsistency this convention replaced was five-line explanations buried
inside test bodies, where they are least likely to be read or maintained.

1. **The why of the story goes in the doc comment of `TestXxxStory`** — what it
   covers, and why these steps belong together.
2. **The what of a step goes in its `t.Run` name.** Write it as a sentence about
   the system: `"refresh rotates, and the token it consumed never works again"`.
3. **The why of an assertion goes in the assertion's own message**, not in a
   comment above it:

   ```go
   assert.Equal(t, http.StatusUnauthorized, unknownEmail.Status,
       "an unknown e-mail must be indistinguishable from a wrong password")
   ```

   This puts the reason in the failure output, where someone reading a red build
   will actually see it.
4. **Inline comments only for what none of the above can carry** — a mechanism
   the reader could not infer, or a setup line whose purpose is not visible.
   Keep them to a line or two.

Same rule in the `internal/` packages, in Go's own idiom: every exported symbol
has a doc comment starting with its name, and the package doc lives in `doc.go`.

## Adding a feature to the suite

1. Regenerate the bindings if the schema changed:
   `scripts/generate-flatbuffers-go.sh`
2. Add the payload builders to `internal/factories/<feature>.go`, valid and
   invalid together.
3. Add `t.Run` steps to the story that owns the resource.
4. Run it: `scripts/integration-test.sh -run TestYardStory`

Each layer's own module documentation says what belongs in it; the
[ADRs](../../docs/adr/) say why.
