# 0010. A published version is a type, and the unversioned path is an alias

**Status:** Accepted · 2026-08-11

## Context

The REST contract had no version at all. Every route lived at the root, so any
change to what a route means — a field renamed, a status changed, a body reshaped
— was a breaking change for every client at once, with no way to ship the new
shape and the old one side by side. The PHP stack had already solved this, and
the Rust port had not brought the solution over.

The obvious cheap answer is a constant: `const VERSION: u16 = 1`, bumped when
something changes. It does not work, because it describes the version rather than
publishing one. Calling a published version *frozen* has to mean that editing it
is not the normal thing to do, and a constant makes editing the only thing to do.

The second question is what the unversioned path should mean. Clients — the front
end, the integration suite, curl in a terminal — already call `/products`, and
breaking all of them to introduce versioning would be the opposite of the point.

## Decision

**A version is a type.** `V1Router` implements `VersionedRouter`, declaring its
number and the full table of what it serves. The next version is a new file
beside it, not an edit to it. Its number is the single source of both the mount
prefix (`/v1`) and the ranking used below; neither is written anywhere else, and
two tables claiming the same number fail the boot.

**A table lists what its version serves in full, not a delta.** A route absent
from `V2Router` is simply not served under `/v2`, and that absence is how a route
that only ever existed in v1 gets written.

**The unversioned path is served too, and resolves per route.** `RouterHub`
mounts each version under its prefix and then walks the versions newest-first,
registering the first occurrence of each `(method, path)` pair at the root. So
`/products` reaches the newest version that still publishes it — decided route by
route, not globally. A route carried from v1 into v2 answers from v2 at the root;
one dropped in v2 keeps answering from v1.

The identity is the method **and** the path, and both halves matter. It is
exactly the pair axum refuses to see twice — a repeat panics with "Overlapping
method route" rather than letting the second win — so the duplicate has to be
gone *before* registration. And it is per-verb because a `GET` carried forward
and a `DELETE` left behind on the same path must be able to resolve to different
versions.

## Consequences

Every route is now written once and mounted twice, which doubles nothing in
source but does mean `VersionedRouter::routes` is called twice per boot. It
builds handler closures, not connections; the cost is boot-time and small.

The root is a **convenience, not a contract**. What it points at moves the day a
new version publishes the same route. A client that means v1 should ask for
`/v1`, which is what `swagger.json` says in its `servers` block. This is a real
sharp edge and it is deliberate: the alternative — freezing the root at v1
forever — would leave every existing caller on the oldest contract by default.

Shipping v2 means writing `V2Router` and adding one line to `RouterHub::build`,
above the v1 line. Forgetting the ordering does not fail loudly; it silently
makes the older version win at the root. That is the weakest point of the design,
and the reason the ordering is stated in the doc of the function rather than left
to be inferred.

## Revisit if

A version ever needs to differ from another by more than its route table — a
different auth scheme, a different wire format, a different base path. At that
point the trait is carrying one decision (the table) while the differences live
elsewhere, and separate routers assembled independently would be the honest
shape.
