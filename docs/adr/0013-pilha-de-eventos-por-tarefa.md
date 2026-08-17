# 0013. A task-scoped event stack, next to the tracing span

**Status:** Accepted · 2026-08-16

## Context

A use case sometimes knows something the edge needs, and there is no channel
between them.

The case that forced this: seven read paths answer from the view cache
(`self.views.get(...)` across six services). A hit and a miss return
byte-identical payloads, so nothing outside the process can tell them apart — not
a client, and not the Go integration suite, which is the only thing that
exercises the cache at all. It could assert neither that the cache was working
nor which of the two answers it had just received.

The obvious channels are all worse than they look.

**Return it.** `get` would answer `(View, WasCached)` instead of `View`. The
controller then has to thread that second value through to the response builder,
the wire DTO has to grow a field the client never asked for, and every future
event repeats the whole exercise. The signature of a use case would come to
describe the middleware stack.

**Put it on the request.** Stamping a header on the way in for a later layer to
read is our own data taking a round trip through a client-controlled structure —
the same objection that already keeps `X-Request-Id` from being trusted inbound.

**Use the tracing span.** This was the closest call, because the project already
decided that context crossing layers rides on the span the middleware opened,
specifically to avoid wrapping the request in a second scope. Two things make it
the wrong tool here:

- A span field is fixed at macro expansion. `Span::record` with a key that was
  not in the macro is a silent no-op — the failure mode is "nothing happens, no
  error", which is exactly what a cache-status header must not do.
- The subscriber blocks reentrancy: inside a subscriber callback,
  `Span::current()` returns no span. The span is written to be *read by something
  else* — a log aggregator, later. What is needed here is a typed value the same
  process reads back, within the same request, to decide what to answer.

They are different mechanisms with different jobs, and the second one did not
exist.

## Decision

A **task-scoped event stack**, in `crates/app/src/event/`.

A `MetaEvent` is anything a service needs to tell a middleware. Meta, not domain:
it describes the *path the request took inside*, never what happened in the yard.
If a client needs the fact as business information, it belongs in a View instead.

The stack is split into two contracts, and the split is the point:

- `MetaEventStackPublisher::emit` is **`pub(crate)` to `app`**. Only a use case
  emits, and nothing outside the crate can even name the contract it would have
  to implement or receive.
- `MetaEventStackSubscriber` — `scope`, `flush`, `captured` — is **public**,
  because the layer that opens the scope and the layer that reads it are the
  presentation.

Two middlewares, deliberately not one. `MetaEventLayer` opens the scope and does
nothing else; `CacheStatusLayer` asks whether `ViewCacheHit` was recorded and
stamps `Cache-Status` (RFC 9211). Merging them would make the opener know about
an event it has no reason to know, and a new event would touch it.

Every operation is idempotent with respect to the scope existing: outside one,
`emit` and `flush` do nothing and `captured` is `false`.

## Consequences

- **Adding an event touches two files.** A variant here, an `emit` in the use
  case that knows the fact. No signature between them changes, and no middleware
  that does not care finds out.
- **The stack is a `Cell<u8>` — a bitmask, one bit per event.** No allocation, no
  `RefCell`, no runtime borrow tracking. The ceiling is eight events today, and
  the width is private to `intern/meta_event_stack.rs`: nothing else — not the
  enum, not the traits, not the middlewares — can see it.
- **The ninth event will not compile.** `bit()` is an exhaustive `match`, so a new
  variant fails the build in the one file that knows the mask width. Widening
  `u8` to `u16` is then a deliberate one-line edit, rather than a bit silently
  shifting out of the byte.
- **Multiplicity is gone, and that is a real loss.** Emitting the same event twice
  is indistinguishable from once, and `flush(Some(e))` clears it rather than
  removing one occurrence. Nothing today counts, and everything today asks "did
  this happen?".
- **Layer order matters and fails quietly when wrong.** The reader must be inside
  the scope, so the opener is the outer of the two. Inverted, `captured` answers
  `false` forever and every response is stamped `fwd=miss` — no error, no log.
  That is why the integration test asserts the *hit*, not merely the presence of
  the header, and why the assertion follows a write that guarantees a cold read.
- **`Cache-Status` reports both halves.** Marking only hits would make the absence
  of the header mean "came from the database" and also "this layer fell out of
  the stack". `fwd=miss` separates them.
- **Six service constructors gained a parameter.** Mechanical, and it is the bulk
  of the diff. The alternative — a static namespace, the mould `MasterScope::run`
  already uses — was rejected to keep the dependency visible in the signature,
  which is how everything else in this graph is reached.
- **The use-case tests use the real stack, not a mock.** It is a ZST over a
  task-local, so testing the real thing costs what a double would and proves the
  actual path. No mock was added.
- **There are now two task-scoped mechanisms.** The span carries observation for
  something else to read later; this carries typed facts for this process to read
  now. Anyone adding a third should be made to say which of the two it is.

## Revisit if

An event needs to survive past the response, or to be read by a different task
than the one that emitted it — a background job, a retry, a queue consumer.
Everything here rests on emitter and reader being the same task inside one
request, and that is the fact that would break it.
