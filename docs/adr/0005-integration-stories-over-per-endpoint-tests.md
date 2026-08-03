# 0005. Write integration tests as stories, not one per endpoint

**Status:** Accepted · 2026-07-26

## Context

The integration suite leases an isolated `{API container + database}`
environment per test. Leasing one costs a schema drop, a re-migrate and an API
restart — roughly twenty seconds.

The restart is not negotiable. Dropping the schema also drops the
`ENGINE=MEMORY` registries, and the application fills those exactly once, at
`WorkerStart` — the permissions and the `refresh-token` marker group among them.
Without a restart, every test after the first would run against a server whose
catalogue no longer exists in the database it is talking to.

One test per endpoint would mean paying that twenty seconds per assertion.

## Decision

Group tests into **stories**: one narrative per domain over a single leased
environment, with sub-tests that run in order and share the state each leaves
behind.

Three today — `session` (bootstrap, login, refresh, logout, password change),
`administration` (roles, users, permissions), `yard` (products, containers,
manifests, metrics). Stories run in parallel with each other; sub-tests within
one deliberately do not.

The economics are only half the argument. The other half is that this suite's
real subject is **transitions**. A container cannot be sealed before it is
loaded, cannot be loaded once sealed, cannot be dispatched twice. A refresh
token must stop working after it is spent, and after logout. Those rules are
about order, and a test that rebuilds state from scratch for each assertion can
never observe them.

## Consequences

- A failing step can leave later steps in the same story failing too. The
  sub-test names are written as sentences about the system so the first failure
  identifies itself.
- Sub-tests are coupled by design. Reordering them breaks them — that is the
  property being tested, not an accident.
- Adding an endpoint to an existing resource costs nothing: one more `t.Run` in
  the story that already owns it. Adding a story costs another environment's
  worth of wall time, so it is reserved for a genuinely separate narrative.
- Per-test isolation is not available. Anything needing it is a unit test.

## Revisit if

The reset gets cheap — an API that could rebuild its registries without a
restart would remove the restart, and the calculation changes.
