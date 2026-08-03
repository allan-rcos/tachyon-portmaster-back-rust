# 0003. Use `ENGINE=MEMORY` for the runtime tables

**Status:** Superseded · 2026-07-26

> Superseded by [0009](0009-runtime-registries-in-process.md) · 2026-08-03. The Rust rewrite runs as a
> single multi-threaded process, so the fork boundary that forced these tables
> into the database no longer exists. Kept because the reasoning explains why the
> PHP deployment looked the way it did.

## Context

Three tables — `permissions`, `marker_groups`, `markers` — are unlike the rest
of the schema. Their contents are either rebuilt from the code on every boot
(the metadata registries, see
[0002](0002-metadata-registries-in-the-database.md)) or bounded by a TTL (the
markers). Nothing in them is authored by a user or recoverable only from disk.

They are also the hottest tables in the system: authorization reads
`permissions` on every request, and refresh reads `markers` on every rotation.

Durability for data that is regenerated on startup buys nothing, and the
round-trip cost is what actually shows up.

## Decision

`ENGINE=MEMORY` for all four.

Slug and id are the entire row. A `label` and a `description` column used to
ride along and nothing ever read them — authorization compares slugs and roles
persist slugs. MEMORY pads every `VARCHAR` to its declared width, so those two
columns cost several times the data they annotated, on a table read during every
request. They were dropped. `VARCHAR(64)` is the ceiling the table modules
validate against.

`id` auto-increments, unlike everywhere else in the schema, because it is a
registry index and computing it as `count() + 1` in PHP with four workers
registering concurrently is a race.

`markers` carries no foreign key to `marker_groups`: MEMORY does not support
them. The group is registered at boot before any marker can reference it, so the
constraint holds by construction rather than by the engine.

## Consequences

- **A MariaDB restart empties them.** Metadata re-registers itself at the next
  `WorkerStart`. Markers expiring early only means active sessions must sign in
  again. The table *definitions* survive, so nothing needs re-creating.
- **MEMORY is not transactional.** A `ROLLBACK` does not undo a marker write.
  Acceptable here and nowhere else: the worst case is a token marked consumed by
  a request that then failed, which signs someone out early rather than letting
  a consumed token live on. Nothing in these tables participates in a business
  invariant.
- **MEMORY takes table-level locks.** This shapes the marker code directly:
  reads *filter* on `expires_at > NOW()` instead of deleting what they find
  expired, so a read never takes the write lock. The sweep happens on write,
  where the lock is already held.
- A quiet period would otherwise let expired rows sit in RAM indefinitely, so a
  MariaDB `EVENT` purges hourly as a backstop. That requires the server started
  with `--event-scheduler=ON`; the dev compose stack and the test harness both
  pass it. Reads already filter, so the event is about reclaiming memory, never
  about correctness.

## Revisit if

Anything with a business invariant needs to live in one of these tables. At that
point the table belongs in InnoDB, and the performance argument has to be
re-made from a profile rather than assumed.
