# Release process

The actual sequence used to ship every version so far, including the
parts that aren't obvious from just reading `Cargo.toml`. Written after
doing this four times in one session: 0.1.0 initial release; 0.2.0 (new
`claml`/`linearization` features) alongside new crates
`who-fic-linearization`/`who-fic-claml` at 0.1.0; `who-fic-icd-api` 0.1.0;
and 0.3.0 (a breaking API change, forced by fixing an inconsistency
introduced across the first two rounds — see `AGENTS/lessons.md`)
alongside `who-fic-linearization`/`who-fic-claml`/`who-fic-icd-api` 0.1.1.
Check `tasks.md`'s "Published" sections for the exact current count and
versions — this number will keep growing and isn't worth keeping literally
current here.

## Prerequisites

- `cargo login` credentials for crates.io (this project's maintainer has
  these configured; an agent should not attempt to obtain its own).
- A clean git tree. `cargo publish` refuses to run against uncommitted
  changes (`--allow-dirty` exists but should not be used — commit first).
- `gh auth status` logged in, for verifying CI after pushing (see below).

## Before touching versions

Run the full check list in `AGENTS.md`'s "Build, test, lint" section
against the whole workspace with `--all-features`. Every crate that
changed needs this to be clean before it's publishable.

## Deciding the version bump

Two independent tracks (see `specs/architecture.md`):

- **Lockstep** (`who-fic`, `who-fic-icd`, `who-fic-icf`, `who-fic-ichi`):
  bump `[workspace.package].version` in the root `Cargo.toml` — every
  crate inheriting `version.workspace = true` picks it up automatically.
  Also bump the matching `[workspace.dependencies]` version constraints
  for these crates (each has a line like `who-fic-icd = { version =
  "...", path = "who-fic-icd" }` — update the `version` string to match),
  or the workspace won't resolve.
- **Independent** (`who-fic-linearization`, `who-fic-claml`,
  `who-fic-icd-api`): each has its own `version = "..."` set directly in
  its own `Cargo.toml` (not `.workspace = true`), bumped individually.
  Still add/update its entry in root `[workspace.dependencies]` if other
  crates depend on it.

New backward-compatible public API → minor bump (0.1.0 → 0.2.0 is what
happened when Phase 7 added the `claml`/`linearization` features to the
four lockstep crates). Breaking change → *also* a minor bump, since these
are all pre-1.0 crates (0.2.0 → 0.3.0 is what happened in Phase 12, when
Phase 11's `Index` harmonization changed two crates' `iter()` return
type). Verify with `cargo-semver-checks` before deciding the bump, not
after — it's installed (`cargo install cargo-semver-checks` if it isn't):

```sh
cargo semver-checks check-release -p <crate> --baseline-version <last-published-version>
```

For a pre-1.0 crate, a `y` bump (`0.2.0` → `0.3.0`) is itself sufficient
permission for a breaking change — the tool will report "major change,"
skip its detailed lints, and say "no semver update required," which
means *the bump you already chose already covers whatever changed*, not
that nothing changed. Don't take a clean report as license to skip
picking the bump correctly by hand first.

## The chicken-and-egg publish ordering

This is the part that isn't obvious: crates with path dependencies on
sibling crates in this workspace **cannot be dry-run-published, let alone
published, until those siblings are already live on crates.io** — not
just committed, not just built locally. `cargo publish --dry-run` for a
dependent crate tries to resolve its dependencies against the *real*
crates.io index, and a workspace path-dependency that hasn't been
published yet resolves to nothing.

Concretely, this means publishing happens in dependency order, waiting
for each publish to actually land before starting the next:

```sh
# Example from the who-fic-linearization/who-fic-claml/who-fic-icd-api round:

# 1. Crates with no unpublished workspace deps first.
cargo publish -p who-fic-linearization
cargo publish -p who-fic-claml
# `cargo publish` (non-dry-run) waits for the crate to become available
# on the index before returning — you can Ctrl-C past that wait, but
# there's no reason to; just let it finish.

# 2. Now who-fic-icd's dry-run will actually resolve, because its
#    optional deps on the two crates above are live.
cargo publish -p who-fic-icd --dry-run
cargo publish -p who-fic-icd

# 3. Repeat for who-fic-icf, who-fic-ichi (each depends on
#    who-fic-linearization, now live).
cargo publish -p who-fic-icf
cargo publish -p who-fic-ichi

# 4. Last: who-fic, which depends on all three classification crates.
cargo publish -p who-fic --dry-run
cargo publish -p who-fic
```

If you try to dry-run a dependent crate before its deps are live, you'll
see: `error: failed to prepare local package for uploading / no matching
package named 'who-fic-X' found / location searched: crates.io index`.
That's not a real problem — it's expected, and means "publish the
dependency first, then retry."

## After publishing

1. Verify each crate is actually live (dry-run success and even the
   "Published" message aren't 100% proof the index has propagated
   everywhere):

   ```sh
   cd /tmp && cargo info <crate-name>   # run outside the workspace so it
                                         # can't resolve to the local path
   ```

2. Commit any final `tasks.md`/`CHANGELOG.md` updates recording what was
   published.

3. Push to **all three** git remotes (this repo's `origin` is configured
   to push to GitHub, Codeberg, and GitLab in one `git push`):

   ```sh
   git push origin main
   ```

4. Verify CI actually goes green on GitHub — don't assume it will just
   because local checks passed (see `AGENTS/lessons.md` for why this
   assumption failed silently for a long time in this project):

   ```sh
   gh run list --repo who-fic-rust/who-fic-rust --limit 1
   gh run view <run-id> --repo who-fic-rust/who-fic-rust
   # or wait for it:
   gh run watch <run-id> --repo who-fic-rust/who-fic-rust
   ```

## Licenses in the package

Every crate directory needs its own copy of `LICENSE-MIT`/`LICENSE-APACHE`
(not just at the workspace root) — `cargo package` doesn't reach outside
the crate directory for files by default. New crates must have these
copied in before their first publish, or `cargo publish --dry-run` will
still "succeed" but the published package won't carry them.
