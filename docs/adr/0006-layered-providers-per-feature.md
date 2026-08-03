# 0006. Hand-wired providers, split per feature

**Status:** Accepted · 2026-07-26

## Context

The object graph has to be built somewhere. Under OpenSwoole it has to be built
inside `WorkerStart` — after the fork — so whatever does it runs four times, once
per worker, and the result is per-worker state.

A conventional DI container would resolve this by reflection at runtime. That
buys autowiring at the cost of a resolution step on a path that runs for every
worker, and of a graph nobody can read by opening a file.

The first hand-wired version put everything in one `AppProvider`. It worked, and
it grew past the point of being readable.

## Decision

No container. Each layer has a register that chains downward and a provider that
constructs its own objects:

```
ApiRegister ──► AppRegister ──► InfraRegister ──► DomainRegister
```

Providers memoize (`$this->x ??= new X(...)`), so "one per worker" is the actual
lifetime of everything.

`AppProvider` is split by feature into `App/Interno/Provider/*Provider` — one
class per slice of the surface, each readable on its own, each extending
`FeatureProvider` and re-exported by `AppProvider`. `IAppProvider` stays the
single façade, so nothing outside the layer notices the split.

`FeatureProvider` also builds the permission registrar every guarded use case
needs. The registrar is stateless; what matters is that it writes into
`IInfraProvider::permissionRepository()`, which is memoized per worker — so
however many feature providers exist, they all fill the same registry.

## Consequences

- The wiring is explicit and greppable: to see what a use case is given, open
  the provider that constructs it. There is no reflection and no configuration
  format to learn.
- Adding a dependency means editing the provider. That is real friction, and it
  is the trade being made — the graph stays honest about how large it is.
- A feature touches four providers (`Domain`, `Infra`, `App`, `API`) plus their
  interfaces. The new-feature guide lists all of them so none is forgotten.
- Because everything is built after the fork, nothing can be shared between
  workers by construction. Anything that must be shared goes to the database —
  see [0002](0002-metadata-registries-in-the-database.md).

## Revisit if

The provider classes stop being readable even split per feature. The next step
would be a compiled container — resolved at build time, not per worker — not a
runtime reflective one.
