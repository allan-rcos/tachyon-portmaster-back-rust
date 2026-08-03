# 0002. Keep metadata registries in the database, not in `OpenSwoole\Table`

**Status:** Superseded · 2026-07-26

> Superseded by [0009](0009-runtime-registries-in-process.md) · 2026-08-03. The Rust rewrite runs as a
> single multi-threaded process, so the fork boundary that forced these tables
> into the database no longer exists. Kept because the reasoning explains why the
> PHP deployment looked the way it did.

## Context

Permissions and marker groups are families of code-declared entries, keyed by
slug and registered at boot. They are read on the
authorization path of every request, so they need to be fast.

The first implementation gave each registry its own `OpenSwoole\Table` — shared
memory, no round trip. That was wrong, for a reason worth recording.

The object graph is built inside `WorkerStart`, which runs **after** OpenSwoole
forks. Each worker therefore allocated its own table. The tables were never
shared; there were four of them.

Metadata got away with it. Every worker re-registers the same entries from the
same code, so the four copies agree by construction. But nothing written at
*runtime* could survive that — a marker set by a request on worker 2 was
invisible to worker 3, which is exactly the bug refresh-token revocation would
have had.

## Decision

Store the registries in MariaDB, in `ENGINE=MEMORY` tables, and give markers the
same machinery.

`SqlMetadataRegistry` is the shared base; `PermissionRegistry` and
`MarkerGroupRegistry` supply only `hydrate()`, `label()` and `table()`.

Telemetry events were in this family at first and are not any more. The test for
belonging here is whether the set is *declared* — whether an entry exists only
because some use case said so, leaving it unknowable from the schema. A
permission passes: it exists because a constructor declared it. A telemetry event
does not — the container domain fixes what is worth recording, and a new one is a
code change to `ManifestTM`, not a registration. It is a
`Domain\Enums\TelemetryEvent` now, alongside `ContainerStatus` and `RiskClass`.

Registration is idempotent by slug, and deliberately read-then-insert rather
than an upsert: a slug *is* the whole entry, so a row that already exists is
already correct and there is nothing an upsert could update.

The read-then-insert **is** racy across four workers booting at once, and the
unique index is what settles it. Whoever loses gets a duplicate-key error,
re-reads, and finds the row the winner just wrote. That is the intended path,
not an error path.

Both registries lease a connection from `IPDOPool` directly rather than taking a
transaction session: registration runs at boot, outside any request, so there is
no boundary open for them to enlist in. `SqlQueryRepository` does the same, for
the same reason.

## Consequences

- One shared catalogue instead of four divergent copies, and markers get a
  correct home rather than a broken one.
- The cost is a round trip instead of a shared-memory read. The table is still
  RAM (see [0003](0003-engine-memory-for-runtime-tables.md)), so it is a round
  trip, not a disk write.
- Boot now depends on the database being reachable. A use case that cannot
  register its permission throws, killing the worker — correct, since a worker
  that does not know its own permissions cannot serve anything.
- Restarting the API against a live database is a no-op rather than a conflict,
  because registration is idempotent. Restarting MariaDB alone empties the
  tables and the next `WorkerStart` refills them.

## Revisit if

OpenSwoole gains a table that can be allocated before the fork and shared across
workers, *and* the per-request round trip shows up in a profile. The first
without the second is not a reason.
