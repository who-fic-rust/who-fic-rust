# Tasks: WHO-FIC-Rust

Task breakdown for [plan.md](plan.md). Specs live in [specs/](specs/index.md).

## Phase 0 — Workspace scaffolding

- [x] Root `Cargo.toml` virtual workspace with `members` and shared
      `[workspace.package]` metadata (version, edition, license, repository)
- [x] `.gitignore` (`/target`, `Cargo.lock` kept for now — workspace of libs,
      revisit before publish)
- [x] Crate `who-fic` with placeholder `src/lib.rs` and features
      `icd` / `icf` / `ichi` (all default)
- [x] Crate `who-fic-icd` with placeholder `src/lib.rs`
- [x] Crate `who-fic-icf` with placeholder `src/lib.rs`
- [x] Crate `who-fic-ichi` with placeholder `src/lib.rs`
- [x] `cargo build` and `cargo test` green across the workspace
- [x] Fix copy-paste error in `who-fic-icf/spec/index.md` (named the wrong
      crate and URL)

## Phase 1 — Common core (`who-fic`)

- [x] `FicError` parse-error type (`std::error::Error + Display`), covering
      empty input, bad length, bad character, bad structure
- [x] `Classification` enum (`Icd10`, `Icd11`, `Icf`, `Ichi`),
      `#[non_exhaustive]`
- [x] Document the code-type conventions every subcrate follows
      (`FromStr`, `Display`, `TryFrom<&str>`, `as_str`, ordering, hashing) —
      see [specs/architecture.md](specs/architecture.md)
- [x] `serde` feature that forwards to each enabled subcrate's `serde` feature
- [x] Crate-level rustdoc with a worked example per classification
- [x] Unit tests: feature-gated re-exports resolve (`who_fic::icd::...`, etc.)
- [x] `From<SubcrateError> for FicError` conversions (all four), with tests

## Phase 2 — ICD (`who-fic-icd`)

See [specs/who-fic-icd.md](specs/who-fic-icd.md).

- [x] Module `icd10`: `Icd10Code` — parse/validate `A00`–`Z99` categories with
      optional subdivision (`I63.9`), `category()` / `subdivision()` accessors
- [x] Module `icd10`: `Icd10Chapter` enum (I–XXII) and code→chapter mapping
      (`chapter()` returns `Option<Icd10Chapter>` — WHO reserves some
      numeric sub-ranges within assigned letters)
- [x] Module `icd11`: `Icd11Code` — 4-character stem codes with optional
      subdivisions (`8B20`, `CA40.0`), excluded letters `I`/`O` enforced
- [x] Module `icd11`: extension codes (`X…` prefix) as a distinct type
      (`ExtensionCode`)
- [x] Module `icd11`: `Icd11Chapter` enum (01–26, V, X) and code→chapter
      mapping from the leading character (`Option<Icd11Chapter>`, same
      reasoning as ICD-10)
- [x] Module `icd11`: residual-category detection (`.Y` other-specified,
      `.Z` unspecified)
- [x] Module `icd11`: postcoordination `Cluster`/`ClusterStem` parsing (`/`
      for multiple stems, `&` for stem+extension) — syntax only
- [x] `serde` feature: transparent string serialization, round-trip tests
- [x] Property tests: parse-format round trip; rejection of malformed input
- [x] Rustdoc examples on all public items

## Phase 3 — ICF (`who-fic-icf`)

See [specs/who-fic-icf.md](specs/who-fic-icf.md).

- [x] `Component` enum: BodyFunctions (`b`), BodyStructures (`s`),
      ActivitiesAndParticipation (`d`), EnvironmentalFactors (`e`)
- [x] `IcfCode` — component letter + 1–5 digits; `level()` accessor
      (Chapter / Second / Third / Fourth) and `parent()` navigation
- [x] `Qualifier` — generic scale 0–4, 8 (not specified), 9 (not applicable)
- [x] `QualifiedIcfCode` with component-tagged `QualifierPayload`: component-
      specific qualifier counts and meanings (b: 1; s: up to 3; d: up to 4,
      performance + capacity standard; e: barrier `.n` or facilitator `+n`
      via `EnvironmentalQualifier`)
- [x] `serde` feature + round-trip tests
- [x] Property tests and rustdoc examples

## Phase 4 — ICHI (`who-fic-ichi`)

See [specs/who-fic-ichi.md](specs/who-fic-ichi.md).

- [x] Axis types: `Target` (3 chars), `Action` (2 chars), `Means` (2 chars)
- [x] `IchiCode` — `TTT.AA.MM` parse/format, accessors per axis
- [x] `Section` enum (body systems & functions / activities & participation /
      environment); `section()` returns `Option<Section>` and currently
      always yields `None` — no verifiable Beta-3 leading-character table
      was found (see spec)
- [x] Extension-code groundwork (documentation-only `extension` module)
- [x] `serde` feature + round-trip tests
- [x] Property tests and rustdoc examples
- [x] Document ICHI beta status in crate docs

## Phase 5 — Integration & polish

- [x] Feature-combination build matrix (`cargo hack --feature-powerset`)
      for `who-fic` — all 18 combinations (incl. `serde`) pass
- [x] `serde` round-trip integration tests through the umbrella crate
- [x] Doc tests pass for every crate; intra-doc links resolve
- [x] Verify specs/*.md match implemented behavior; update where they diverge
      (ICD `Option<Chapter>`, `ExtensionCode`/`Cluster`/`ClusterStem` shape,
      `QualifiedIcfCode` representation, ICHI `Section` fallback, ICHI error
      `axis` field)

## Phase 6 — Release readiness

- [x] CI workflow: fmt, clippy `-D warnings`, test, doc, feature matrix, MSRV
- [x] Decide and pin MSRV (`rust-version` in `[workspace.package]`) — 1.85
      (edition 2024's minimum)
- [x] Per-crate README + crates.io metadata (description, keywords,
      categories, docs.rs config)
- [x] CHANGELOG.md
- [x] `cargo publish --dry-run` in dependency order (subcrates, then umbrella)

## Published (0.1.0)

- [x] `who-fic-icd` 0.1.0 — https://crates.io/crates/who-fic-icd
- [x] `who-fic-icf` 0.1.0 — https://crates.io/crates/who-fic-icf
- [x] `who-fic-ichi` 0.1.0 — https://crates.io/crates/who-fic-ichi
- [x] `who-fic` 0.1.0 — https://crates.io/crates/who-fic

## Phase 7 — Data loading (`who-fic-linearization`, `who-fic-claml`)

See [specs/who-fic-linearization.md](specs/who-fic-linearization.md) and
[specs/who-fic-claml.md](specs/who-fic-claml.md). Formats verified against
real WHO downloads on 2026-08-03 (see those specs' "verified" notes).

### `who-fic-linearization` (new crate)

- [x] `LinearizationRow` type with typed accessors for all 13 common
      columns plus the 5 MMS-only `Grouping` columns
- [x] `LinearizationReader<R: Read>` — streaming row iterator, BOM
      stripped, tolerant of short trailing-column lines and mixed
      quoted/bare fields
- [x] `LinearizationError` — `std::error::Error`, reports 1-based line
      number of malformed rows
- [x] `serde` feature
- [x] Tests against hand-written fixtures (not vendored WHO exports) per
      spec's test list

### `who-fic-claml` (new crate)

- [x] `ClamlDocument`/`Class`/`Rubric`/`Label`/`ModifierClass`/`Modifier`
      types per spec
- [x] Parser built on `quick-xml` (the one documented dependency exception
      in the workspace — see specs/architecture.md)
- [x] `Class::preferred_label(lang)` convenience accessor
- [x] `ClamlError` wrapping XML parse errors plus structural problems
      (e.g. `Class` missing `code`)
- [x] `serde` feature
- [x] Tests against a hand-written fixture (not a vendored WHO ICD-10
      export) per spec's test list

### Classification-crate adapters

- [x] `who-fic-icd`: `icd10::claml` module (feature `claml`, dep on
      `who-fic-claml`) — `Icd10ClamlIndex`
- [x] `who-fic-icd`: `icd11::linearization` module (feature
      `linearization`, dep on `who-fic-linearization`) —
      `Icd11LinearizationIndex`
- [x] `who-fic-icf`: `linearization` module (feature `linearization`) —
      `IcfLinearizationIndex`
- [x] `who-fic-ichi`: `linearization` module (feature `linearization`) —
      `IchiLinearizationIndex`
- [x] Each adapter: tests using small hand-written `LinearizationRow`/
      `ClamlDocument` fixtures built from real column/element shapes,
      mapped through to the classification's typed code

### Integration & release

- [x] Add both new crates to the workspace, `cargo build`/`test`/`clippy`/
      `fmt` clean across the whole workspace
- [x] Bump `who-fic-icd`, `who-fic-icf`, `who-fic-ichi`, `who-fic` to 0.2.0
      (new public API, backward compatible — semver minor)
- [x] Update CHANGELOG.md
- [x] `cargo publish --dry-run` for `who-fic-linearization` and
      `who-fic-claml`, then the four 0.2.0 bumps, in dependency order
- [x] Publish all six crates; push to git remotes

## Published (0.2.0 / 0.1.0)

- [x] `who-fic-linearization` 0.1.0 — https://crates.io/crates/who-fic-linearization
- [x] `who-fic-claml` 0.1.0 — https://crates.io/crates/who-fic-claml
- [x] `who-fic-icd` 0.2.0 — https://crates.io/crates/who-fic-icd
- [x] `who-fic-icf` 0.2.0 — https://crates.io/crates/who-fic-icf
- [x] `who-fic-ichi` 0.2.0 — https://crates.io/crates/who-fic-ichi
- [x] `who-fic` 0.2.0 — https://crates.io/crates/who-fic
- [x] `who-fic-icd-api` 0.1.0 — https://crates.io/crates/who-fic-icd-api

## Phase 8 — `who-fic-icd-api`

See [specs/who-fic-icd-api.md](specs/who-fic-icd-api.md). Endpoints and
auth flow verified against WHO's own OpenAPI spec
(`https://id.who.int/swagger/v2/swagger.json`) on 2026-08-04.

- [x] `IcdApiClient`/`IcdApiClientBuilder` with configurable token/API base
      URLs (testability hook)
- [x] OAuth2 client-credentials token fetch + in-memory cache + refresh
- [x] `entity`, `entity_search`, `linearization_entity`, `code_info`,
      `search`, `icd10_category` methods
- [x] `Entity`/`CodeInfo`/`SearchResults`/`Icd10Entity` response types,
      permissive deserialization
- [x] `IcdApiError` (`#[non_exhaustive]`): `Auth`/`Http`/`Status`/`Decode`
- [x] Typed-code convenience for `Icd10Code`/`Icd11Code`
      (`code_info_typed`/`icd10_category_typed`)
- [x] Tests against a local `wiremock` server (no live WHO credentials
      available to this workspace)
- [x] README + rustdoc examples
- [x] `cargo publish --dry-run`, then publish; push to git remotes

## Backlog / future subcrates (not scheduled)

- [ ] Semantic cluster validation for ICD-11 postcoordination (needs WHO data)
- [ ] Split `who-fic-icd` into `who-fic-icd-10` / `who-fic-icd-11` if the
      revisions grow enough to justify it
