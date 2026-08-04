# AGENTS.md

Operational guide for AI coding agents (and humans) working in this
repository. This file is a **guide**; `specs/architecture.md` and the
other `specs/*.md` files are the **authoritative specification** — when
they disagree, the specs win and this file should be corrected to match.

## What this is

A Cargo workspace implementing the WHO Family of International
Classifications (FIC) for Rust: seven published crates covering ICD-10,
ICD-11, ICF, and ICHI code parsing/validation, two shared parsers for
WHO's export formats, and an async client for WHO's live ICD-API. Full
context: [plan.md](plan.md) (goals, design principles, phase history),
[tasks.md](tasks.md) (what's shipped, what's queued), [specs/](specs/index.md)
(the specification, one file per crate plus a cross-cutting
`architecture.md`).

**Read `specs/architecture.md` before making any non-trivial change.** It
covers the code-type conventions, error shape, data-loading index
conventions, dependency policy, and — most important — the licensing
constraint below.

## The licensing constraint (read this first)

WHO owns the copyright on classification *content*: code titles,
definitions, inclusion/exclusion notes, the complete tabular lists. **This
repository must never vendor or embed that content.** What's in scope:
code syntax/structure/validation, small individual codes as factual
examples in docs/tests (already done throughout — `A00`, `b280`,
`KAB.DB.AD`, etc. are fine), and parsers for formats the *user* supplies
their own WHO-obtained file/credentials for. If a change would require
bundling WHO's actual data (full code lists, titles, descriptions), it
does not belong in this repository — see `specs/architecture.md`'s
"Licensing constraint" section.

## Spec-driven development: specs/ is the single source of truth

The workflow this repository follows, and that you should follow:

1. **Before implementing**, check whether `specs/*.md` already describes
   the behavior. If it doesn't, or the change diverges from what's
   written, update the spec first (or alongside the implementation) —
   don't let code and spec drift.
2. **After implementing**, verify the spec still matches reality exactly:
   method signatures, error variants, feature names, examples. A spec
   that describes an API the code doesn't have (or vice versa) is worse
   than no spec — the whole point is that an agent (or human) can trust
   `specs/*.md` without re-reading every implementation file.
3. **When adding a new crate**, write its `specs/<crate-name>.md` first
   (see the existing ones for the expected shape: syntax grammar, types,
   errors, `serde`, tests), then implement to match it, then add it to
   `specs/index.md`'s spec map.
4. **This has already gone wrong once** — see `AGENTS/lessons.md` for the
   concrete case (four data-loading `Index` types built independently by
   different agents drifted from each other because no single agent had
   read all four; harmonized in Phase 11, now documented once in
   `specs/architecture.md` instead of four times).

## Build, test, lint

Exactly what CI runs (`.github/workflows/ci.yml`) — run the same commands
before considering work done:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo test --workspace                              # default features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo hack check -p who-fic --feature-powerset       # cargo install cargo-hack if missing
cargo audit                                          # cargo install cargo-audit if missing
cargo deny check                                     # cargo install cargo-deny if missing
```

**Gotcha, learned the hard way**: `cargo build -p <crate> --no-default-features`
(lib only) is *not* the same check as `cargo test --workspace
--no-default-features` (also compiles examples and tests). CI failed
silently on every push for this repository's first several commits
because local verification only ever ran the former. Always run the exact
`cargo test`/`cargo build` invocation CI runs, for the whole workspace,
not an approximation scoped to the crate you touched.

MSRV is 1.85 (edition 2024's minimum), pinned in the root `Cargo.toml`'s
`[workspace.package]`.

## Architecture at a glance

```
who-fic-rust/ (virtual workspace)
├── who-fic              umbrella crate (icd/icf/ichi/serde/claml/linearization features)
├── who-fic-icd          ICD-10 + ICD-11 code types (modules: icd10, icd11)
├── who-fic-icf          ICF code types
├── who-fic-ichi         ICHI code types
├── who-fic-linearization  parser: WHO's shared TSV export format (ICD-11/ICF/ICHI)
├── who-fic-claml        parser: ClaML XML (ICD-10)
└── who-fic-icd-api      async client for the live WHO ICD-API
```

- `who-fic`, `who-fic-icd`, `who-fic-icf`, `who-fic-ichi` version together
  (currently 0.2.0). `who-fic-linearization`, `who-fic-claml`,
  `who-fic-icd-api` version independently (currently 0.1.0 each).
- `who-fic-icd`/`who-fic-icf`/`who-fic-ichi` optionally depend on
  `who-fic-linearization`/`who-fic-claml` (feature-gated: `claml`,
  `linearization`) to add data-loading `*Index` types. `who-fic-icd-api`
  depends on `who-fic-icd` directly (non-optional) for typed-code
  convenience methods.
- Every classification code type follows the same trait set
  (`FromStr`/`Display`/`TryFrom`/`as_str`/ordering) — see
  `specs/architecture.md`'s "Code-type conventions".
- Every data-loading `*Index` type follows the same shape (`BTreeMap`-backed,
  entries carry their own code, `iter()`/`IntoIterator` in ascending order)
  — see `specs/architecture.md`'s "Data-loading index conventions".

## Deeper topics

- **[AGENTS/release.md](AGENTS/release.md)** — the actual publish
  process: version bumping, dry-run, the chicken-and-egg dependency
  ordering, pushing to three git remotes, verifying CI after.
- **[AGENTS/testing.md](AGENTS/testing.md)** — testing conventions per
  crate kind (pure parsers, format parsers, network clients), the
  `wiremock` pattern for `who-fic-icd-api`, property testing.
- **[AGENTS/lessons.md](AGENTS/lessons.md)** — specific mistakes made and
  caught during this project's development, and what to do differently.
  Read this before large multi-crate changes.

## Repository conventions

- Dual-licensed MIT / Apache-2.0. Copy `LICENSE-MIT`/`LICENSE-APACHE` into
  any new crate directory (crates.io packaging needs them locally, not
  just at the workspace root).
- Every crate has `#![warn(missing_docs)]` in `lib.rs`; combined with
  clippy's `-D warnings`, an undocumented public item fails CI.
- Every crate has a `README.md` with badges (crates.io/docs.rs/CI/license)
  and an `examples/readme.rs` mirroring its README's code sample — keep
  them in sync, and verify the example actually runs
  (`cargo run --example <name> -p <crate> --all-features`) rather than
  trusting that copy-pasted code compiles.
- If a crate's `examples/` directory would produce a target name that
  collides with another crate's (e.g. every crate having `readme.rs`),
  give it an explicit `[[example]] name = "..."` in `Cargo.toml` — see any
  existing crate's `Cargo.toml` for the pattern. This bit CI once already.
- No vendored WHO content, ever (see above). No new dependencies without a
  reason tied to what the crate specifically needs (see
  `specs/architecture.md`'s "Dependencies" section for the two documented
  exceptions and why they're exceptions, not precedent).
