# 0004. Bootstrap through `POST /setup`, not a SQL-seeded user

**Status:** Accepted · 2026-07-26

## Context

A fresh deployment has no users. Every route that creates one sits behind a
permission, and no one holds any permission yet — so there has to be exactly one
way in.

The obvious answer is to seed an administrator in SQL. That was the first
implementation, and it required the seed file to carry two things it had no
business carrying: a pre-computed argon2id hash, and a hand-copied list of
permission slugs for the administrator role.

Both rot silently. The hash would have become wrong the day the hashing
parameters changed, with nothing to notice. The slug list had already drifted
three slugs behind the code before anyone spotted it — precisely because nothing
exercised it.

## Decision

A single endpoint, `POST /setup`, creates the first user and an `Administrator`
role in one transaction.

Its guard is not a permission but `IUserRepository::hasAny()` — it refuses with
409 once any user exists. That is the right guard, not a missing one: the
endpoint's precondition is genuinely "this system is empty", which no permission
can express.

The role is built from `IPermissionRepository::all()`, the live registry, rather
than a literal list. A permission introduced by a future use case is therefore
granted here without anyone remembering to come back and add it — the registry
is filled at `WorkerStart` by the use cases themselves.

Everything happens inside one boundary, so a deployment cannot end up with a
role and no user, or a user who owns nothing.

The dev seed and the integration harness both bootstrap this way. `db/seeds/dev.sql`
seeds products and containers only.

## Consequences

- The bootstrap path a real deployment uses is the one the tests exercise, on
  every single environment reset. It cannot drift, because nothing else exists.
- No credential material and no permission list in version control.
- The endpoint is unauthenticated by necessity. It is a one-shot: the second
  caller gets a 409, which the session story asserts explicitly.
- Someone must call it after a deployment. That is a documented step, in
  `docs/infrastructure.md` and `db/README.md`.

## Revisit if

Multi-tenancy arrives, where "the system has no users" stops being a global
property and the `hasAny()` guard no longer means what it says.
