# Architecture Decision Records

Decisions whose *reasoning* would otherwise have to be reverse-engineered from
the code. Each records what was decided, what it cost, and what would have to be
true to revisit it.

These exist because the reasoning used to live in twenty-line comment blocks at
the top of classes, where it made the class docblock unreadable and the rendered
API documentation useless. The code now states what it does; these state why.

| # | Decision |
|---|---|
| [0001](0001-flatbuffers-over-json.md) | FlatBuffers as the wire format, with hand-written proxies |
| [0002](0002-metadata-registries-in-the-database.md) | Metadata registries in the database, not `OpenSwoole\Table` *(superseded by 0009)* |
| [0003](0003-engine-memory-for-runtime-tables.md) | `ENGINE=MEMORY` for permissions and markers *(superseded by 0009)* |
| [0004](0004-setup-endpoint-instead-of-a-seeded-user.md) | `POST /setup` instead of a SQL-seeded first user |
| [0005](0005-integration-stories-over-per-endpoint-tests.md) | Integration tests as stories, not one per endpoint |
| [0006](0006-layered-providers-per-feature.md) | Hand-wired providers, split per feature |
| [0007](0007-phpstan-baseline-limited-to-generated-code.md) | A PHPStan baseline holding only generated code |
| [0008](0008-minified-tarball-as-the-release-artifact.md) | A minified tarball as the release artifact, migrations apart |
| [0009](0009-runtime-registries-in-process.md) | Runtime registries in process; InnoDB rather than MyRocks |
| [0010](0010-versions-as-types-with-an-unversioned-alias.md) | A published version is a type; the unversioned path is a per-route alias |
| [0011](0011-static-providers-one-per-directory.md) | Providers are static struct-namespaces, one per constructing directory *(supersedes 0006 for the Rust port)* |

## Writing one

Only when a future reader would otherwise be puzzled — not for routine work. Use
the next free number and keep the four headings:

```markdown
# NNNN. Title in the imperative

**Status:** Accepted · YYYY-MM-DD

## Context
What forced a choice.

## Decision
What was chosen.

## Consequences
What it costs, including what is now harder.

## Revisit if
The condition that would make this wrong.
```

Then link it from the table above, and from the docblock of the code it
explains:

```php
 * @see docs/adr/0003-engine-memory-for-runtime-tables.md Why this table is in RAM.
```
