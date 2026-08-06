# Spec: Architecture and conventions

Workspace-wide decisions that every crate in this repository follows.

## Workspace

The repository root is a virtual Cargo workspace (no root package):

```toml
[workspace]
resolver = "3"
members = [
    "who-fic", "who-fic-icd", "who-fic-icf", "who-fic-ichi",
    "who-fic-linearization", "who-fic-claml", "who-fic-icd-api",
]
```

Shared metadata lives in `[workspace.package]` (version, edition, license,
repository, authors, rust-version) and each crate inherits it with
`field.workspace = true`. Intra-workspace dependencies are declared once
in `[workspace.dependencies]` (path + version, so published crates
resolve on crates.io) and referenced with `workspace = true`.

Two version tracks:

- **Lockstep**: `who-fic`, `who-fic-icd`, `who-fic-icf`, `who-fic-ichi`
  version together — they shipped together in Phase 0-1, gained new
  features together in Phase 7, and took a breaking change together in
  Phase 11/12, so they bump together. The current value is
  `[workspace.package].version` in the root `Cargo.toml`; this file
  deliberately does not repeat the number (it went stale twice when it
  did).
- **Independent**: `who-fic-linearization`, `who-fic-claml`,
  `who-fic-icd-api` — added later (Phases 7-8), each versions on its own
  schedule since they have their own release cadence. The current values
  are in each crate's own `Cargo.toml` (mirrored in the root
  `[workspace.dependencies]` pins for the two that other workspace crates
  depend on).

- Edition: 2024
- License: MIT OR Apache-2.0 (code only; see "Licensing" below)
- MSRV: 1.85, pinned via `rust-version` in `[workspace.package]` (edition
  2024's minimum) — every crate inherits it, and CI's `msrv` job runs
  `cargo check --workspace --all-features` on that exact toolchain (a
  compile check only — it does not run tests on 1.85).

New subcrates are added as workspace members named with their parent
crate's name as prefix — e.g. `who-fic-icd-api` (the WHO ICD-API client,
prefixed under `who-fic-icd` since it's that classification's live-data
companion). See `tasks.md`'s "Backlog" section for what's queued but not
yet started.

## Code-type conventions

Every classification code type in every crate (`Icd10Code`, `Icd11Code`,
`ExtensionCode`, `IcfCode`, `QualifiedIcfCode`, `IchiCode`, and the ICHI
axis types `Target`/`Action`/`Means`):

(`who-fic-icd`'s `Cluster`/`ClusterStem` are *not* code types in this
sense and are deliberately outside this convention: `Cluster` implements
`FromStr`/`TryFrom`/`Display`/`Hash` but has no `as_str()` and no
ordering, and `ClusterStem` is a read-only component of a parsed
`Cluster`, constructible only through `Cluster` parsing — see
`specs/who-fic-icd.md`.)

1. Is an owned, immutable value type wrapping validated content; construction
   only via parsing/`TryFrom` — no public constructor that skips validation.
2. Implements `FromStr` (the canonical parser), `TryFrom<&str>`,
   `Display` (the canonical formatter), `Debug`, `Clone`, `PartialEq`, `Eq`,
   `Hash`, `PartialOrd`, `Ord`.
3. Provides `as_str(&self) -> &str` returning the canonical string form.
4. Round-trips: `T::from_str(s).unwrap().to_string()` equals the canonical
   form of `s`, and parsing the canonical form is identity.
5. Parsing is case-insensitive on input where the classification prints
   uppercase (ICD, ICHI) — canonical form is uppercase. ICF component letters
   are lowercase by convention and canonicalized to lowercase.
6. Whitespace is not trimmed by parsers; callers trim. (Keeps parsers exact
   and composable.)

**Validation depth:** parsers validate *syntax and structure* (allowed
characters, lengths, axis/qualifier structure, chapter ranges). They do not
validate *existence* (whether WHO has assigned the code) — that requires WHO
data and belongs to the data-loading features and `who-fic-icd-api` (see
"Licensing constraint" below).

## Data-loading index conventions

`who-fic-icd` (`icd10::claml`, `icd11::linearization`), `who-fic-icf`
(`linearization`), and `who-fic-ichi` (`linearization`) each expose an
optional-feature-gated `*Index` type (`Icd10ClamlIndex`,
`Icd11LinearizationIndex`, `IcfLinearizationIndex`,
`IchiLinearizationIndex`) that adapts parsed rows/classes from
`who-fic-linearization`/`who-fic-claml` into a lookup keyed by that
classification's typed code. All four follow one shared shape:

- Backed by a `BTreeMap` (not `HashMap`) — every code type already
  implements `Ord`, and a deterministic, sorted iteration order is more
  useful than an unspecified one for a "browse everything" use case, at
  no extra cost.
- The paired `*ClassEntry` type (`Icd10ClassEntry`, `Icd11ClassEntry`,
  `IcfClassEntry`, `IchiClassEntry`) always carries its own `code()`
  accessor, in addition to `title()` and a raw class-kind string (plus
  classification-specific extras, e.g. `Icd11ClassEntry`'s
  `is_residual()`/`chapter_no()`/`groupings()`). One naming divergence
  survived the Phase 11 harmonization: `Icd10ClassEntry` calls the
  accessor `kind()` (ClaML's attribute name), the other three call it
  `class_kind()` (the linearization column name). Renaming either way is
  a breaking change and has not been judged worth it.
- `Index::iter(&self) -> impl Iterator<Item = &ClassEntry>` and
  `impl<'a> IntoIterator for &'a Index` (same `Item`) both yield entries
  only, in ascending code order — not `(code, entry)` tuples, since the
  entry is already self-describing via `.code()`.
- `Index::title(&self, code) -> Option<&str>`,
  `Index::get(&self, code) -> Option<&ClassEntry>`,
  `Index::len(&self) -> usize`, `Index::is_empty(&self) -> bool`.
- Construction (`from_document`/`from_rows`) is lenient about individual
  rows/classes that don't parse as that classification's code type (e.g.
  a ClaML chapter/block class, an ICHI proposed entry with a placeholder
  target) — those are silently skipped, not treated as a build failure.
  For the three linearization indexes, a malformed *source file* (the
  underlying reader erroring mid-stream) *is* a build failure, propagated
  through that crate's own `*Error` type via `From`.
  `Icd10ClamlIndex::from_document` is the asymmetric fourth: it takes an
  already-parsed `&ClamlDocument`, so file-level parse errors surface
  earlier, from `who-fic-claml` itself, and `Icd10ClamlError` neither
  wraps `ClamlError` nor is in practice ever returned (its own rustdoc
  says so; the `Result` signature matches the other three indexes).

This shape was harmonized across all four in Phase 11 — the four
were originally built independently and drifted (mixed `HashMap`/`BTreeMap`,
mixed tuple-vs-entry `iter()`, inconsistent `IntoIterator` presence) since
no single agent saw all four at once. If you add a fifth data-loading
index, match this shape.

## Errors

- `who-fic` defines `FicError`, the shared parse-error enum
  (see [who-fic.md](who-fic.md)).
- Subcrates each define their own error type (e.g. `Icd10ParseError`) so they
  stay independent of the umbrella crate; variants follow the same shape
  (empty input / invalid length / invalid character with position / invalid
  structure with reason).
- All error types implement `std::error::Error + Display + Debug + Clone +
  PartialEq` and are `#[non_exhaustive]`, with two exceptions:
  `who-fic`'s `ParseClassificationError` (a struct, shipped without
  `#[non_exhaustive]`; its one field is private, so the attribute would
  change nothing for downstream code anyway) and `who-fic-claml`'s
  `ClamlError` (no `PartialEq`, because it wraps `quick_xml::Error`,
  which has none).
- `who-fic-linearization`, `who-fic-claml`, and `who-fic-icd-api` are not
  WHO-FIC *code* crates, so their error types (`LinearizationError`,
  `ClamlError`, `IcdApiError`) don't need to match this shape
  variant-for-variant, but still follow the same spirit: `#[non_exhaustive]`,
  `std::error::Error + Display + Debug`. The three linearization-backed
  `*Index` error types wrap `LinearizationError` via `From`;
  `Icd10ClamlError` does not wrap `ClamlError` (see the construction
  asymmetry under "Data-loading index conventions" above).

## Features

Per subcrate:

- `serde` — adds `Serialize`/`Deserialize` to the crate's public data
  types. Codes serialize as their canonical string (via
  `Display`/`FromStr`), not as structs; enums (`Component`, `Qualifier`,
  `Section`, chapters, …) and `*ClassEntry` types use plain derives
  (variant/field names, not canonical strings).
  `who-fic-linearization`'s `LinearizationRow` and `who-fic-claml`'s
  document types are plain data structs — no single canonical string to
  round-trip through — so they derive directly too. Not covered by the
  feature anywhere: error types, and `who-fic-icd`'s
  `Cluster`/`ClusterStem` (no serde impls at all).
  `who-fic-icd-api` is the odd one out: it has no `[features]` table —
  `serde` is a required dependency there and its response types are
  `Deserialize`-only.
- `claml`/`linearization` — the data-loading features described above,
  each gated on the matching format-parser crate as an optional
  dependency.
- `std` is not a feature: crates target `std` for now. `no_std` (with
  `alloc`) is a possible later addition; avoid gratuitous `std`-only
  dependencies in core code to keep the door open.

Umbrella crate `who-fic`:

- `icd`, `icf`, `ichi` — each pulls in and re-exports the matching subcrate.
  All three are default features.
- `serde`, `claml`, `linearization` — each forwards to every enabled
  *classification* subcrate's matching feature (`"who-fic-icd?/serde"`
  syntax). The classification crates' own `serde` features in turn
  weakly forward to their optional parser deps
  (`"who-fic-linearization?/serde"`, `"who-fic-claml?/serde"`), so
  `serde` + `linearization`/`claml` through the umbrella makes
  `LinearizationRow`/`ClamlDocument` serializable too (a real gap
  through 0.3.1, fixed in 0.3.2). `who-fic`'s own
  `Classification`/`FicError` types have no serde impls.

## Dependencies

- Core crates (`who-fic`, `who-fic-icd`, `who-fic-icf`, `who-fic-ichi`,
  `who-fic-linearization`): zero required dependencies. `serde` (with
  `derive`) optional.
- No `regex` dependency for code parsing — the grammars are small and fixed;
  hand-written matchers are faster to compile, dependency-free, and give
  better error positions. `who-fic-linearization`'s tab/quote-aware field
  scanner follows the same principle for its (also small, fixed) format.
- Dev-dependencies may include `proptest` for property tests,
  `serde_json` for serde round-trip tests, and (in `who-fic-icd-api`
  only) `wiremock` plus `tokio` with `rt-multi-thread`/`macros` for the
  mock-server tests.
- Heavier dependencies only in the two crates whose entire purpose requires
  them — documented here so they don't look like drift from the
  zero-dependency norm elsewhere:
  - `who-fic-claml` depends on `quick-xml`, because general XML parsing is
    not a good fit for a hand-written matcher the way the small
    fixed-grammar codes elsewhere in this workspace are.
  - `who-fic-icd-api` depends on `who-fic-icd` (non-optional, for the
    typed-code convenience methods — the one intra-workspace hard
    dependency outside the umbrella), `reqwest` (`default-features =
    false`, features `json` + `rustls`; the `rustls` backend links no
    OpenSSL — its `rustls-native-certs` transitively pulls
    `openssl-probe`, which only *locates* system CA certificates),
    `serde`/`serde_json`, and `tokio`
    (the `sync` feature only, for the token cache's `Mutex`; it does not
    bundle a runtime — callers supply their own; `rt-multi-thread`/`macros`
    are dev-dependency-only, for tests and examples), because it is the
    one crate in the workspace whose job is making real HTTP calls to
    WHO's live API. No other crate needs, or should reach for, these
    dependencies.
- Dependency versions are checked to be current (`cargo update
  --dry-run`) and audited (`cargo audit`, `cargo deny check` against
  `deny.toml`) — see "CI" below.

## Testing

- Unit tests per module: accept-lists of known-good codes, reject-lists of
  known-bad codes (with the expected error variant).
- Property tests: parse→format→parse round trip; arbitrary strings never
  panic the parser.
- Doc tests: every public item has at least one rustdoc example that
  compiles and runs. `who-fic-icd-api`'s top-level client example is the
  one documented exception — it needs live WHO credentials and network
  access, so it's marked `no_run` rather than executed.
- `who-fic-icd-api`'s test suite runs entirely against a local `wiremock`
  mock server — no live WHO credentials or network access required.
- Umbrella crate: build/test under the feature power set (`cargo hack
  --feature-powerset` in CI, or an explicit matrix).
- `#![warn(missing_docs)]` is set in every crate's `lib.rs`; combined with
  clippy's `-D warnings` in CI, an undocumented public item fails the
  build.

## CI

GitHub Actions (`.github/workflows/ci.yml`) on push/PR, one job per
concern: `fmt` (`cargo fmt --all --check`), `clippy` (`cargo clippy
--workspace --all-targets --all-features -- -D warnings`), `test` (matrix
over `--all-features` / `--no-default-features` / default features),
`feature-powerset` (`cargo hack check -p who-fic --feature-powerset`),
`doc` (`cargo doc --workspace --all-features --no-deps` with
`RUSTDOCFLAGS="-D warnings"`), `msrv` (`cargo check --workspace
--all-features` on the pinned 1.85 toolchain), `audit`
(`rustsec/audit-check`), `deny` (`EmbarkStudios/cargo-deny-action`
against `deny.toml`).

CI was not actually verified to pass on GitHub for the first several
commits of this repository's history — it silently failed on every push
until Phase 9 hardening caught and fixed the underlying bugs (an
unguarded example under `--no-default-features`, and output-filename
collisions across crates' identically-named examples). Lesson: `cargo
build -p <crate> --no-default-features` (lib only) is not the same check
as `cargo test --workspace --no-default-features` (also compiles
examples/tests) — verify the exact command CI runs, not an approximation
of it.

Standing rule from the collision fix: any crate whose `examples/` file
would produce a target name already used by another crate (every crate's
`readme.rs`) declares an explicit unique `[[example]] name =
"who-fic-<crate>-readme"` in its `Cargo.toml`. `who-fic-icd-api` needs no
alias — its example filenames are already unique.

## Licensing constraint (important)

WHO owns the copyright on classification *content* (code titles,
descriptions, inclusion/exclusion notes, the complete tabular lists).
This repository must not vendor or embed that content. What ships here:

- Code **syntax** rules (formats, alphabets, lengths, separators)
- **Structural** facts (components, axes, qualifier scales, chapter ranges
  and their leading characters)
- Small numbers of individual codes in tests and docs as factual examples

Anything requiring the full classification data (titles, existence checks,
search) is out of scope for the core crates and is delegated to:

- The format-parser crates (`who-fic-linearization`, `who-fic-claml`) and
  each classification crate's optional data-loading feature (see
  `specs/who-fic-icd.md`, `specs/who-fic-icf.md`, `specs/who-fic-ichi.md`,
  and "Data-loading index conventions" above) — the *user* supplies a WHO
  export file they obtained themselves; these crates parse it, they do
  not fetch or embed it.
- `who-fic-icd-api` (see `specs/who-fic-icd-api.md`) — the *user* supplies
  their own WHO ICD-API credentials; this crate is a thin wrapper around
  WHO's live REST API, it does not cache or bundle response content
  beyond a process-lifetime in-memory OAuth2 token.

## Related documents

For AI coding agents working in this repository: `AGENTS.md` (and
`CLAUDE.md`, which imports it) is the operational entry point — build/test
commands, the spec-driven-development workflow, and pointers into
`AGENTS/*.md` for deeper topics (release process, this file's conventions
restated more tersely). This file (`specs/architecture.md`) is the
authoritative *specification*; `AGENTS.md` is a *guide* to working in the
repository that references it.
