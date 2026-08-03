# 0009. Keep the runtime registries in process, and InnoDB for everything else

**Status:** Accepted · 2026-08-03

Supersedes [0002](0002-metadata-registries-in-the-database.md) and
[0003](0003-engine-memory-for-runtime-tables.md).

## Context

Three tables — `permissions`, `marker_groups`, `markers` — existed in the
database for one reason, recorded in 0002: the PHP runtime forked four
OpenSwoole workers, each with its own memory. A permission registered by worker 2
was invisible to worker 3, and a refresh token revoked on one worker stayed live
on the others. The database was the only ground the four workers shared, and
`ENGINE=MEMORY` (0003) was how that shared ground avoided a disk write.

The Rust rewrite is **one process with many threads**. An `Arc<Cache<..>>`
allocated at boot is reachable from every task on every thread, by construction.
The premise both ADRs rested on is gone.

Separately, the workspace notes named MariaDB MyRocks as the storage engine. The
official `mariadb:11` image (11.8.8) ships 29 plugins and `ha_rocksdb.so` is not
among them; MyRocks would mean a custom image or a distribution change, for a
workload of six tables where nothing points at write amplification as a problem.

## Decision

**The registries move into the process.** `permissions` and `marker_groups`
become two Moka caches held by the infra provider; `markers` becomes a third,
with the TTL the engine already supports. The three tables and the hourly purge
`EVENT` are gone from the schema, and the dev stack and test harness no longer
need `--event-scheduler=ON`.

**Each registry gets its own map, with its own type.** They started as one shared
`Arc<Cache<String, ()>>` — one keyspace for both vocabularies — and the marker
group `refresh-token` promptly turned up in `GET /metadata/permissions` *and* in
the role that `POST /setup` grants, which receives everything registered. The
maps are now distinct newtypes, so handing a registry the wrong one does not
compile. The old schema kept them in two tables and never had the problem; the
type is what restores that separation.

**InnoDB for the six remaining tables.** Not MyRocks.

## Consequences

- **No round trip on the authorization path.** Every request checked
  `permissions` against the database before; now it reads a map. This was the
  cost 0002 accepted knowingly, and it is what disappears.
- **Boot no longer needs the database to register metadata** — though it still
  needs it, for everything else.
- **Markers are lost on restart**, as before. A restart signs everyone out of
  their *refresh* session; the access tokens outlive it until they expire. Same
  behaviour as a MariaDB restart under the old design.
- **The registries do not survive a process, and there is exactly one process.**
  A second replica behind a load balancer would each hold its own catalogue —
  fine, since the catalogue is code — but each would also hold its own markers,
  and a refresh rotated on replica A would be unknown to replica B. This is the
  real limit of the decision, and it is written into "revisit if" below.
- **`ROLLBACK` still does not undo a marker write.** The cache is not
  transactional, exactly as MEMORY was not. Nothing in it participates in a
  business invariant.
- The `EVENT` and its scheduler flag are gone: Moka expires entries itself.

## Revisit if

The API is deployed as more than one process. At that point markers need shared
storage again — Redis rather than a MEMORY table, since the requirement is a
shared TTL map and not a relation — and the `cache-redis` feature already
carved out for it is where that goes. The permission catalogue does **not**
follow: it is built from code, so every replica derives the same one.

Or, for the engine: if a profile shows write amplification on `telemetry_logs`
mattering, MyRocks is worth the custom image. Not before.
