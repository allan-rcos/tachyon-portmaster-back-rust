# 0008. Ship a minified tarball, with the migrations packaged apart

**Status:** Accepted · 2026-08-01

## Context

Until 1.0.0 there was no way to produce a deployable copy of this application.
Deploying meant cloning the repository on the server and running
`composer install` there, which puts a toolchain, a set of credentials and a
network dependency on a machine that needs none of the three.

The obvious alternative — a Docker image — already exists and stays. What was
missing is the case where the target is a plain server: an OpenSwoole process
that *is* the web server has nothing to gain from a container runtime underneath
it, and a 970 KB tarball is a materially simpler thing to move, verify and roll
back than an image registry.

The two obstacles were size and safety. A production `vendor/` is 14.2 MB
before pruning, and the usual way to shrink PHP — stripping comments — is only
sound if nothing reads them back at runtime.

## Decision

`scripts/build-dist.sh` produces two zstd tarballs in `dist/`, each with a
`.sha256`:

| Artifact | Holds | Applied by |
|---|---|---|
| `portmaster-api-<version>-linux-x86_64.tar.zst` | `src/` + `vendor/`, pruned and minified | the server |
| `portmaster-migrations-<version>.tar.zst` | `db/migrations` and `db/seeds` | a developer, against the database |

Three choices inside it are the ones worth recording.

**The migrations do not travel with the API.** The machine running the API is
the one least entitled to alter the schema, and a deployment that can migrate is
a deployment that can migrate by accident. Splitting the packages makes applying
a migration a separate, deliberate act by someone who can see the database.

**`php -w` is guarded, not assumed.** It strips comments and whitespace while
preserving semantics, which holds only while nothing in the codebase is driven
by annotation. That is true today — there is no `getDocComment` and no
`Reflection*` in `src/`, and the only `__DIR__` is the autoload `require` in
`src/API/main.php`, which the strip does not move across lines. The script
re-checks this on every build and **fails** if it stops being true. The cost of
being wrong here is not a build error but an API that boots in production with
an attribute that quietly vanished, so the check belongs where it cannot be
skipped.

**`LICENSE` files survive the prune.** Removing them saves nothing measurable
and breaks the redistribution terms of roughly half the dependency tree.

The build is assembled in `dist/.stage`, never in the working copy. Composer
runs with `--working-dir` against a staging tree that already holds `src/`,
`composer.json` and `composer.lock`. The straightforward version of this script
runs `composer install --no-dev` at the repository root, which deletes PHPStan,
Pest and Mockery from the working copy of whoever ran it — a footgun that only
fires locally, never in CI, which is the worst place for it to hide.

Measured at 1.0.0: `src/` 1.3 MB → 491 KB, `vendor/` 14.2 MB → 5.4 MB over 772
minified files, yielding a 982 KB API tarball and a 5.5 KB migrations tarball.

## Consequences

- A stack trace from production points at line 1 of everything. The `.sha256`
  identifies exactly which commit's output is deployed, and the unminified tree
  is a `git checkout` away, but reading a trace now requires that extra step.
- Any future annotation-driven library — a DI container reading attributes, a
  serialiser reading docblocks — is a build failure rather than a silent
  breakage. Adopting one means dropping `php -w`, not working around the guard.
- The artifact is `linux-x86_64` by name but not by content: it is pure PHP, and
  the platform in the filename records the musl target it was built and tested
  against, not a hard constraint.
- `ext-openswoole` is installed on no build machine, since
  `--ignore-platform-req` skips it. The build therefore cannot detect an
  extension-level incompatibility; only the runtime can.

## Revisit if

The deployment target becomes containers only, in which case the image is the
artifact and this script has nothing left to do. Or if `src/` ever grows a
dependency on runtime reflection over docblocks — then the guard fires, and the
choice is between dropping minification and keeping the constraint.
