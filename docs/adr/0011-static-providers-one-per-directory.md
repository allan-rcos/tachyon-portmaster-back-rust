# 0011. Providers are static, one per directory

**Status:** Accepted · 2026-08-16

## Context

[0006](0006-layered-providers-per-feature.md) settled the shape for the PHP
stack: hand-wired providers, one per layer, chained downward. The Rust port
brought it over and added a trait per layer — `DomainProvider`, `InfraProvider`,
`AppProvider`, `ApiProvider` — each with RPITIT factories returning `impl Trait`.

The trait turned out to cost more than it bought.

**It made the provider an object, and the object had to be carried.** To reach a
`UserTM` you first had to hold the thing that knows how to build one. The
`P: AppProvider` parameter travelled through `router`, `RouterHub::build`,
`VersionedRouter::routes` and `V1Router::routes` — four generic signatures whose
only job was to pass an object along until the route table could ask it for ten
controllers. `ApiProviderImpl` carried an `app: P` field for the same reason.

**It made configuration a struct field.** `DomainProviderImpl` stored
`DomainSecrets` because one factory out of eleven needed it. `ApiProviderImpl`
stored `ApiConfig` and `JwtConfig` because two of sixteen did. Everything else
carried them.

**And it hid a real bug.** `DomainProviderImpl::database_id_generator()` built a
`SnowflakeIdGenerator` on *every* call, so `user_table_module()` and
`role_table_module()` got independent generators sharing one instance id. A
Snowflake id is timestamp, instance and a per-generator sequence; two generators
with the same instance have independent sequences, so within the same
millisecond both emit sequence 0 — the same id. The file's own doc said as much,
and nothing enforced it.

The trait was never what made the graph substitutable, either. What a test swaps
is the *dependencies* a provider injects, and those are still traits.

## Decision

Providers are **static struct-namespaces**: a unit type with associated
functions, the same mould as `CacheLimits` and `SystemLogger`. No trait, no
instance, no field.

There is one per **directory that constructs something**, and the one above
encapsulates the ones below until the layer boundary:

```
DomainProvider ──► TableModulesProvider ──► IdProvider, SecurityProvider
InfraProvider  ──► RepositoryProvider   ──► MariaDb…, Memory…
                   QueryProvider, LoggingProvider, ScopeProvider
AppProvider    ──► ServicesProvider
ApiProvider    ──► ControllersProvider  ──► TokenProvider, MiddlewareProvider
```

Only the layer provider is `pub`. Directories that hold contracts and data —
`commands/`, `queries/`, `error/`, `enums/`, `views/`, `wire/` — get none, since
there is nothing to build.

**Implementations lose their `new`.** Each file exports one function that builds
the struct; the struct itself becomes private, so the concrete type has no name
outside the directory it is born in. The function returns `impl Trait` unless the
provider above it has to store what it built — a `static` needs a name — and even
then the name goes no further than that directory.

**Configuration is installed into the provider that consumes it**, and nowhere
else. Nothing stores the config itself: it is consumed on the spot by the object
that needs it, and what stays behind is the **object**. No struct field anywhere.

**One method per value**, named after what it swaps:
`DomainProvider::install_identity`, `InfraProvider::install_database`,
`ApiProvider::install_environment`, `ApiProvider::install_jwt`. Four values, four
methods. A single `install` taking everything would force whoever wants to change
the environment name to hold the signing key as well, which is the coupling this
whole ADR is about removing — at a smaller scale, but the same kind.

Installing again **replaces**. That is the difference between configuration and a
resource: a pool must not be opened twice by accident, but a config value is
meant to be changeable, and a process that can be pointed at another database or
handed a new signing key without a restart is worth more than one that silently
ignores the second call. The `RwLock` that holds each of them is what makes the
swap possible; the hot path is `read`, and what comes out of it is a clone.

**Only what is expensive or what configuration built is memoized.** That is five
things: the connection pool, the four Moka maps, the Snowflake generator, the JWT
service and the environment name. Nothing else. Everything else is rebuilt on
every call, including the Argon2 hasher — what is expensive about Argon2 is
deriving a hash, which happens per verification either way; *building* one reads
build constants.

Rebuilding is cheap here because of where the graph is consumed. Controllers are
built once, when the route table is assembled, and cloned per request — so the
table modules, repositories and services underneath them are built once at boot
too, not per request. Nothing in the request path constructs anything.

The three that must be stored are stored for three different reasons, and it is
worth keeping them apart:

- The **pool** is a resource. A second one would double the open connections
  against a server with a ceiling on them.
- The **Moka maps** *are* the state. A cache rebuilt per call is not a cache.
- The **Snowflake generator** is a correctness constraint. Two generators with
  the same instance have independent sequences, so within one millisecond both
  emit the same id.

Memoizing more than that was tried and rejected. A `static` needs a **nameable**
type, and a table module is `UserTMImpl<G, H>`: naming it means naming the
concrete type of every helper it holds, which drags those types out of the module
that owns them and undoes the boundary `impl Trait` draws. Across crates it is
not merely undesirable but impossible — an object built in `infra` from something
the `domain` returned as `impl Trait` has a type parameter with no name at all.
Letting the storage mechanism dictate the type discipline was the wrong trade.

Not memoizing has a second payoff: a config swap propagates precisely *because*
the graph above is rebuilt. Install a new pool and the next repository built
already talks to it, with no invalidation to write and nothing to walk.

## Consequences

- **The id collision is closed.** No test covers it: tests in this repository are
  for use cases and `TableModules` only, and a provider is neither. It was
  verified by a throwaway test that was run and deleted.
- **The presentation layer lost four generic signatures.** `router`,
  `RouterHub::build`, `VersionedRouter::routes` and `V1Router::routes` take no
  type parameter, and `api-http`'s `bootstrap/` went from three files to one.
- **`register` is gone.** It existed to construct the provider and stash the
  secrets. There is no provider to construct, and the secrets are installed into
  whoever consumes them — so what was left of it became methods on the layer
  provider: `install`, `InfraProvider::check_database` and `AppProvider::boot`,
  which chains the two and declares the permission catalogue.
- **Some factories return `anyhow::Result`, and deliberately not all.** Whatever
  depends on the pool can fail before boot has supplied the database secrets;
  whatever only touches memory cannot. A uniform signature would demand `?` from
  calls that cannot go wrong. Every call happens at boot, inside functions that
  already returned `anyhow::Result`.
- **Every provider method still returns `impl Trait`.** The `static` holds the
  concrete type, which it must, but that type never appears in a provider's
  signature: the function that builds it is private to its directory, and what
  the provider hands out is the contract. Memoizing more than the leaves would
  have cost exactly this, which is the reason it is not done.
- **A swap does not reach what already holds a clone.** A repository mid-query
  finishes on the old pool, which closes when its last clone drops; the
  `SessionLayer` built into the router keeps accepting tokens signed by the
  previous key until the router is rebuilt. Both are the honest behaviour of a
  handle that was handed out, and neither is reachable by accident — the only
  caller of `install` is boot.
- **The tower layers keep their `new` and return concrete types.**
  `Router::layer` constrains `L::Service`, and an `impl Layer<Route>` hides that
  associated type behind an opaque one, which cannot carry bounds. This is the
  one exception in the graph, and it belongs to axum.
- **Nothing is substitutable at the provider level any more.** That is the trade,
  and it costs nothing today: no test ever built a provider — they build the
  implementation directly with mocks, which is what the generics are for.

## Revisit if

Two graphs are needed *at the same time* in one binary — two pools, two signing
keys, live together rather than one replacing the other. One slot per process is
the assumption the whole design rests on, and that is the fact that would break
it. Swapping is already covered; coexisting is not.
