# 0012. The driver alone, whole queries, and time as an integer

**Status:** Accepted · 2026-08-16

## Context

`sqlx` was three things at once in this codebase: a MariaDB driver, a connection
pool, and a `QueryBuilder`. Only the first was ever used the way the library
offers it.

**The pool was already wrapped.** `MariaDbUnitOfWork` owns the handle, the
transaction lives in a task-scoped slot behind an `OwnedMutexGuard`, and nothing
outside `scope/database/` ever sees a pool. Whatever sqlx offered there arrived
already hidden.

**The builder was writing SQL anyway.** Every `build` was a run of
`builder.push("…literal SQL…")` calls with `push_bind` between them. The builder
contributed no schema knowledge, no type checking and no composition — it was a
`String` with an awkward API and a bind counter. Two queries needed real
assembly (a conditional filter and a variable-length `IN`), and both are three
lines of `format!`.

**The compile-time checking was not in use.** `query!`/`query_as!` need a live
database or a checked-in `.sqlx/` cache; the project has neither, and migrations
are applied by golang-migrate. Every query went through the runtime-checked
`query_as`, so the type safety being paid for was the type safety of any driver.

Two smaller problems were visible in the same code and cost nothing extra to fix
while it was open.

**SQL lived in `const` items, often in pieces.** `COLUMNS`, `JOIN_ROLES`,
`USER_COUNT` — half-queries at the top of a file, imported *across* files:
`get_account`'s projection was also `list_users`'s, and `list_containers`'s was
also `get_container`'s and `list_container_summaries`'s. Reading any one of those
queries meant assembling it from two files, and changing one silently changed the
other.

**Time was stored as `DATETIME`, and written by the server.** The columns carried
`DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP`, so the instant recorded
was when the `INSERT` reached MariaDB — decided by the server's clock and its
session time zone. Meanwhile the domain models already stamped `Utc::now()` on
construction and on every mutation, and the repositories dutifully copied those
values into an entity that then dropped them on the floor. The whole arrangement
depended on the database session being pinned to `+00:00`, which is why three
separate files — the pool, the compose stack, the Go test harness — carried a
paragraph explaining the pinning.

## Decision

**`mysql_async` 0.37, and nothing in place of the rest.** The driver is
tokio-native, its pool is lazy (so opening still touches no network and `open`
stays synchronous), and `start_transaction` yields a `Transaction<'static>`,
which is exactly what the existing task-scoped slot already stored.

No query builder — not `sea-query`, not one of our own. SQL is written as SQL.

**A query is written whole, in the method that runs it.** No SQL in a `const`,
and no fragment shared between files. Concatenation is allowed and expected —
a conditional filter has to be assembled somehow — but it happens inside `build`
or inside the repository method, where the reader can see the entire statement.
What stays shared is *hydration* (`read_item`, `read_role_of`), which is row
reading, not querying.

**Parameters are named (`:last_id`), not positional.** A value that appears twice
in the text — the search filter, which the page and its `COUNT(*)` both apply —
is bound once. The driver repeats it at every occurrence, ignores map entries the
text does not mention, and fails only when the text names something the map
lacks.

**Every temporal column is `BIGINT` holding epoch milliseconds**, written by the
application. No `DATETIME`, no `TIMESTAMP`, no column default, no `NOW()`,
`UTC_TIMESTAMP()` or `UNIX_TIMESTAMP()` in any statement. The translation to
`DateTime<Utc>` happens once, in `entity/decode.rs`; the domain, the services and
the wire see exactly what they saw before.

The database had not reached production, so migration `000001` was edited in
place rather than corrected by a `000002`.

## Consequences

- **One dependency where there were two layers of one.** `mysql_async` with
  `default-features = false, features = ["default-rustls-ring"]`. The default
  feature set pulls `flate2/zlib` — zlib in C — and the Dockerfile's build stage
  has no C toolchain on purpose, which is why `jsonwebtoken` runs on
  `rust_crypto`. `cargo tree -i` finds no `libz-sys`, no `openssl-sys` and no
  `native-tls`. The `chrono` feature is off: no column is a date, so the driver
  never needs to know the type.
- **Row → entity is still a derive.** `FromRow` from `mysql_common`, with
  `#[mysql(json)]` for `permissions` and a `deserialize_with`/`serialize_with`
  pair for the four types that cannot be built from a `Value` on their own.
  Those pairs live in `entity/decode.rs`, the counterpart to `Codec`.
- **`UserEntity` writes its `FromRow` by hand.** The derive has no `skip`, and
  `roles` is not a column — it comes from a second query. Fifteen lines, and the
  entity is still the row: no `*_row.rs` came back.
- **`serialize_with` is only needed by a field that precedes another.** The
  derive calls it when rebuilding the row for a `FromRowError`, and only for
  fields already read. `deleted_at` is last everywhere, so it reads without a
  return path; adding a field after it is a compile error, not a silent gap.
- **A missing parameter name fails at runtime, not at compile time.** That is the
  price of having no builder. The mitigation is that text and values are built in
  the same `if`, and that the integration suite exercises every filter
  combination. A *spare* entry in the map is harmless — the driver ignores it —
  so the failure mode is one-directional.
- **Three files lost a paragraph about time zones.** The pool still runs
  `SET time_zone = '+00:00'` on every new connection, and the compose stack and
  the Go harness still pass `--default-time-zone=+00:00`, but all three are now
  nets rather than mechanisms: nothing stored depends on them. They are kept so a
  `NOW()` typed into a shell answers the way the application would.
- **The seeds carry a literal epoch.** They relied on the column defaults and now
  write `1735689600000` explicitly. Fixed rather than "now", so re-running the
  seed converges on the same database.
- **The summary query lost two conversions.** `CAST(UNIX_TIMESTAMP(t.timestamp)
  * 1000 AS SIGNED)` inside the `JSON_OBJECT` is just `t.timestamp`: the column
  holds the number the View wants.
- **The JSON aggregation stayed exactly as it was.** One round trip per page of
  container summaries instead of the 2n a per-container query would cost, the
  `COALESCE(… LIMIT 1 OFFSET 9)` recency window included. It is the one place
  where SQL does real work, and none of this made it simpler.
- **Anyone with an existing dev database needs `docker compose down -v`.** The
  migration changed under them. The Go harness rebuilds the schema per story and
  does not care.
- **Nothing versioned tests this layer.** Tests cover use cases and
  `TableModules`; what proves this change is the Go integration suite, plus two
  manual checks — that boot fails on a bad password at `ping` rather than on the
  first request, and that a timestamp survives a round trip through the API.

## Revisit if

Queries start needing to be *composed* rather than written — a filter grammar
built at runtime, or a projection assembled from user input. Everything here
rests on a query being a fixed piece of text with holes in it, and that is the
fact that would stop being true.
