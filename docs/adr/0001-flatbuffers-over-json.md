# 0001. FlatBuffers as the wire format, with hand-written proxies

**Status:** Accepted · 2026-07-26

## Context

The API's payloads are structured, repetitive and read far more often than they
are written. JSON means parsing every field of every response on both sides,
and a contract that exists only as prose.

FlatBuffers gives a schema that generates code for every language that needs it
— here PHP for the API and Go for the integration suite — and access without a
parse step.

The cost is that the generated PHP is not good. flatc's PHP backend has
long-standing defects that leak wrong types into whatever calls it, and the
generated classes are plain data accessors with no place to put anything the
application needs.

## Decision

Schemas in `swagger/flatbuffers/schemas/*.fbs`, in a submodule shared with
whatever else consumes the contract. `flatc` generates the tables.

**Generated code is never edited.** Each generated table has a hand-written
`*Proxy` beside it — `ProductResponse` and `ProductResponseProxy` — extending it
and adding what the application actually uses: a typed constructor, `buildInto`,
`toBinary`, and JSON coercion via the `CoercesJson` trait. The rest of the
codebase talks to proxies exclusively.

Content negotiation is a middleware (`FlatBufferNegotiationMiddleware`), so the
same endpoint serves binary or JSON from `Accept` and `Content-Type`. Clients
that cannot do FlatBuffers are not shut out, and the tests can read a response
by hand when debugging.

flatc's defects are corrected by `scripts/patch-flatbuffers.php`, run
immediately after generation by the `composer flatbuffers` script. Its
transforms are deterministic and idempotent, so they survive regeneration:

1. Builder class casing — upstream declares `FlatbufferBuilder`, flatc
   references `FlatBufferBuilder`, which fails PSR-4 autoloading on
   case-sensitive filesystems.
2. `create<Table>()` return docblocks — flatc annotates `@return <Table>` on a
   method that returns an `int` offset, poisoning every caller's inferred type.
3. Absent child tables — flatc emits `: 0` where an object is expected,
   producing a `<Child>|int` union; rewritten to `: null`.

## Consequences

- One schema drives PHP and Go, and CI fails if the committed Go bindings are
  stale — the contract cannot silently diverge between the API and its tests.
- Two files per message instead of one. The proxy is where all the hand-written
  behaviour goes, so this is a filing cost, not a duplication one.
- Generated files are excluded from PHPStan analysis (see
  [0007](0007-phpstan-baseline-limited-to-generated-code.md)) and from the
  rendered documentation. Proxies are excluded from neither.
- A schema change is a four-step operation: edit `.fbs`, `composer flatbuffers`,
  `scripts/generate-flatbuffers-go.sh`, update the proxies. The new-feature
  guide walks it.
- `flatc` is required to change schemas, though not to run anything.

## Revisit if

flatc's PHP backend becomes good enough that the proxies have nothing left to
add, or the negotiation middleware shows that no client ever asks for binary.
