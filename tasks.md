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
      for `who-fic` — the full powerset of its then-four features
      (`icd`/`icf`/`ichi`/`serde`) passed; the powerset has since grown
      with `claml`/`linearization` (64 combinations today) and CI runs it
      on every push
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
      (produced by Phase 8, below — listed here so all
      published-versions blocks stay complete)

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

## Phase 9 — Hardening (2026-08-04)

CI had never actually been verified to pass on GitHub since the first
commit — this phase found and fixed that, plus a few other rough edges.

- [x] **Fixed CI, which had been failing on every push since day one**:
      `who-fic/examples/readme.rs` referenced `who_fic::icd`/`icf`/`ichi`
      unconditionally, breaking `cargo test --workspace
      --no-default-features` (feature-gated the example body)
- [x] Fixed output-filename collisions across all six crates' identically
      named `examples/readme.rs` (Cargo warning, "may become a hard error
      in the future") by giving each an explicit unique `[[example]]` name
- [x] Bumped `actions/checkout@v4` → `v5` in CI, clearing Node.js 20
      deprecation warnings on every run
- [x] Added crates.io/docs.rs/CI/license badges to all seven crate
      READMEs; rewrote the previously-7-line root README into a proper
      workspace index
- [x] Closed a `#![warn(missing_docs)]` enforcement gap: `who-fic-icd`,
      `who-fic-ichi`, and `who-fic-linearization` lacked it (the other
      four crates had it); no items were actually undocumented, but future
      ones would have silently passed CI
- [x] Audited every `.unwrap()`/`.expect()`/`panic!()` in non-doctest
      library code workspace-wide: all are provably-safe internal
      invariants (e.g. guarded by a preceding `is_empty()`/`find()`
      check); none are latent panics on untrusted input
- [x] Checked for `TODO`/`FIXME`, `#[ignore]`d tests, and clippy
      `#[allow(...)]` suppressions: none found beyond one well-justified,
      already-commented exception
- [x] Confirmed the source files with no inline `#[test]` module (12 of
      them, recounted in the Phase 16 audit — this entry originally said
      7) are covered by rustdoc doctests instead (a deliberate,
      already-passing strategy), except
      `who-fic-ichi/src/extension.rs`, which is a documentation-only
      stub by design

## Phase 10 — Dependency auditing, repo hygiene, dogfooding example (2026-08-04)

- [x] `cargo audit`: 0 vulnerabilities across 213 dependencies
- [x] `cargo deny` (`deny.toml` added): populated the license allow-list
      (MIT, Apache-2.0, ISC, BSD-3-Clause, Unicode-3.0, CC0-1.0, MIT-0,
      Unlicense, CDLA-Permissive-2.0 — everything actually in the tree,
      nothing extra); advisories/bans/licenses/sources all pass clean
- [x] Wired both into CI as new `audit`/`deny` jobs so this runs on every
      push/PR, not just once locally
- [x] `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1),
      GitHub issue templates (bug report, feature request), PR template
- [x] Dogfooding example: `who-fic-icd-api/examples/lookup_code.rs` chains
      `who-fic-icd` (offline parse + chapter lookup) with
      `who-fic-icd-api` (live WHO lookup) — the two crates' intended
      division of labor in one runnable example

## Phase 11 — Cross-crate harmonization, agent docs, tutorial (2026-08-04)

- [x] Audited all four data-loading `*Index` types side by side (they'd
      been built independently by different agents) and harmonized:
      `BTreeMap`-backed (was 3× `HashMap` + 1× `BTreeMap`), `ClassEntry`
      always carries `code()` (was missing on 2 of 4), `iter()`/
      `IntoIterator` yield entries in ascending code order everywhere
      (was inconsistent tuple-vs-entry, and `IntoIterator` missing on 3
      of 4) — documented once as a shared convention in
      `specs/architecture.md` instead of four times
- [x] Full `specs/*.md` reconciliation pass: `architecture.md` had drifted
      significantly (stale workspace member list, "Phase 6" framing for
      long-shipped work, no mention of the three newer crates,
      `missing_docs`, `deny.toml`, or the `audit`/`deny` CI jobs) and got
      a comprehensive rewrite; `who-fic-icd-api.md`'s method list was
      missing `entity_search`
- [x] Dependency versions checked current (`cargo update --dry-run`): all
      pinned versions already latest, nothing to bump
- [x] `AGENTS.md` (root, canonical) + `CLAUDE.md` (imports it via
      `@AGENTS.md`) + `AGENTS/release.md` / `AGENTS/testing.md` /
      `AGENTS/lessons.md` — operational guides for AI coding agents,
      capturing real incidents from this project's development (the
      silently-failing-CI gotcha, the publish dependency-ordering
      chicken-and-egg, the Index-type drift) rather than restating specs
- [x] `TUTORIAL.md` — guided walkthrough tying all seven crates together;
      every code snippet verified to actually compile against the real
      APIs (extracted into scratch examples, built, deleted) before
      being committed
- [x] `specs/index.md` and root `README.md` updated to cross-link the new
      docs

## Phase 12 — Version bump and republish for Phase 11's breaking change (2026-08-04)

Phase 11's `Index` harmonization changed `IcfLinearizationIndex`/
`IchiLinearizationIndex`'s `iter()`/`IntoIterator` item type — a breaking
change to the already-published 0.2.0. Pre-1.0 semver: breaking change
bumps the minor version.

- [x] Lockstep group (`who-fic`, `who-fic-icd`, `who-fic-icf`,
      `who-fic-ichi`): 0.2.0 → 0.3.0
- [x] `who-fic-linearization`, `who-fic-claml`, `who-fic-icd-api`: 0.1.0 →
      0.1.1 (non-breaking additions only — example-collision fix, a new
      example file)
- [x] CHANGELOG.md updated with the breaking-change note
- [x] Full workspace verification, publish in dependency order, push

## Published (0.3.0 / 0.1.1)

- [x] `who-fic-linearization` 0.1.1 — https://crates.io/crates/who-fic-linearization
- [x] `who-fic-claml` 0.1.1 — https://crates.io/crates/who-fic-claml
- [x] `who-fic-icd` 0.3.0 — https://crates.io/crates/who-fic-icd
- [x] `who-fic-icf` 0.3.0 — https://crates.io/crates/who-fic-icf
- [x] `who-fic-ichi` 0.3.0 — https://crates.io/crates/who-fic-ichi
- [x] `who-fic` 0.3.0 — https://crates.io/crates/who-fic
- [x] `who-fic-icd-api` 0.1.1 — https://crates.io/crates/who-fic-icd-api

## Phase 13 — Accuracy pass on plans/tasks/files/agent files (2026-08-04)

- [x] Fixed two genuinely broken install snippets (`TUTORIAL.md`,
      `who-fic/README.md` still pinned `"0.2"`, which can't resolve to
      0.3.0)
- [x] Fixed `specs/architecture.md`'s stale "currently 0.2.0" version
      claim, adding a pointer at `Cargo.toml` as source of truth — but
      kept a literal number alongside, which promptly went stale again
      after Phase 15's bump; the numbers were finally removed outright in
      Phase 16
- [x] Fixed three files (`specs/architecture.md`, `AGENTS.md`,
      `AGENTS/testing.md`) that misattributed the Index harmonization to
      "Phase 9" (actually Phase 11)
- [x] Found and corrected a fabricated claim in `AGENTS/testing.md`
      (asserted a specific bug was "caught during harmonization" — checked
      the actual test modules and found it false; all four adapters
      already had the relevant tests before harmonization)
- [x] `AGENTS/release.md` claimed "no breaking change shipped yet" — ran
      `cargo-semver-checks` (installed earlier, never used) against the
      0.2.0 baseline to validate the 0.3.0 bump decision with real tool
      output instead of an untested claim
- [x] `plan.md`'s status line and phase list stopped at "Phase 8 in
      progress"; added Phases 9–12 summaries
- [x] Added missing `## Install` sections to the six individual crate
      READMEs (previously only `TUTORIAL.md` had one under that name;
      `who-fic`'s README covers installation under its `## Usage`
      heading, which it keeps); verified all seven version requirements
      resolve against the live registry by building a throwaway project
      against them

## Phase 14 — More examples, FAQ, tutorial expansion (2026-08-04)

- [x] One new, distinct runnable example for each of the six non-umbrella
      crates (not a README mirror): `postcoordination_cluster.rs` (icd),
      `hierarchy_walk.rs` (icf), `axis_composition.rs` (ichi),
      `stream_and_filter.rs` (linearization), `walk_hierarchy.rs`
      (claml), `search_and_traverse.rs` (icd-api) — each built/run, and
      those six crates' READMEs link to their `examples/` directories
      (`who-fic` itself kept just its readme example and no link)
- [x] `FAQ.md`: which-crate-do-I-need, offline-vs-live title lookup, why
      no WHO content is bundled, syntax-vs-existence validation, explicit
      non-endorsement for clinical/billing/regulatory use, the
      linearization-crate vs. `*Index`-adapter distinction, `chapter()`
      returning `Option`, async scope, ICHI beta-status parsing
      surprises, `no_std` status, MSRV, pre-1.0 semver policy
- [x] `TUTORIAL.md`'s data-loading section: replaced "downloaded
      yourself" with the actual verified download steps (URL, filenames,
      ZIP contents) for ICD-11/ICF/ICHI; honest caveat that ICD-10 ClaML
      sourcing wasn't verified to the same precision
- [x] Rustdoc annotation audit (84 candidate items + full module-doc
      sweep): concluded existing coverage is already correctly weighted,
      no changes needed — verified against `LinearizationRow`'s
      type-level example and confirmed the one module needing a `//!` doc
      (a `pub mod`, not a private implementation detail) already had one

## Phase 15 — Republish for Phase 13/14 content (2026-08-04)

Every crate's packaged content changed since the 0.3.0/0.1.1 publish (new
`examples/*.rs`, updated `README.md`) but no `src/` files changed in any
crate — confirmed via `git diff --stat` per crate directory before
deciding the bump. Non-breaking: patch bump across the board.

- [x] Lockstep group: 0.3.0 → 0.3.1
- [x] Independent crates: 0.1.1 → 0.1.2
- [x] Full workspace verification, publish in dependency order, push

## Published (0.3.1 / 0.1.2)

- [x] `who-fic-linearization` 0.1.2 — https://crates.io/crates/who-fic-linearization
- [x] `who-fic-claml` 0.1.2 — https://crates.io/crates/who-fic-claml
- [x] `who-fic-icd` 0.3.1 — https://crates.io/crates/who-fic-icd
- [x] `who-fic-icf` 0.3.1 — https://crates.io/crates/who-fic-icf
- [x] `who-fic-ichi` 0.3.1 — https://crates.io/crates/who-fic-ichi
- [x] `who-fic` 0.3.1 — https://crates.io/crates/who-fic
- [x] `who-fic-icd-api` 0.1.2 — https://crates.io/crates/who-fic-icd-api

## Phase 16 — Comprehensive audit and doc reconciliation (2026-08-06)

Workspace-wide audit: every claim in `specs/*.md` compared against the
implementation (five parallel review passes), and `plan.md`/`tasks.md`
compared against git history, CI, and the crates.io state. Code health
re-verified first: fmt, clippy `-D warnings`, and the full test suite all
green; CI green on the latest push; no vendored WHO content anywhere
(fixture sweep re-confirmed).

- [x] `plan.md`: brought the status line current (it said "12 phases,
      0.3.0/0.1.1" — two releases and four phases behind), replaced
      embedded version numbers with pointers to `Cargo.toml`, added
      `who-fic-icd-api` to the architecture tree and dependency graph
      (its non-optional dep on `who-fic-icd` was missing), added Phase
      13–16 summaries, carved the backlog out of the definition of done
- [x] `tasks.md`: fixed wrong counts ("18 combinations" → the actual
      then-16/now-64 powerset; "7 source files" → 12; "6-line" README →
      7), corrected the Phase 13 entries that overstated what was done
      (the architecture.md version rewording that kept a stale number;
      the `## Install` claim about `who-fic`'s README), scoped Phase 14's
      "per crate" example claim to the six crates it actually covered
- [x] `specs/architecture.md`: removed the twice-stale version numbers;
      documented the real exceptions to the error conventions
      (`ParseClassificationError` not `#[non_exhaustive]`, `ClamlError`
      not `PartialEq`, `Icd10ClamlError` not wrapping `ClamlError`); made
      the CI job list match `ci.yml` exactly (incl. `msrv` being `cargo
      check`, not a build); scoped the serde-feature claim to what
      actually derives it; completed the `who-fic-icd-api` dependency
      list (`who-fic-icd`, reqwest `json` feature); recorded the
      `[[example]]` naming rule, the `kind()`/`class_kind()` naming
      split, and `Cluster`/`ClusterStem` being outside the code-type
      conventions
- [x] `specs/who-fic.md`: documented the serde-forwarding gap (see
      backlog), the real error-shape divergences the `From` impls paper
      over, `ParseClassificationError`, the all-hyphens `FromStr`
      leniency, and the single (not per-classification) crate-doc example
- [x] `specs/who-fic-icd.md`: corrected the module tree (was missing
      `ExtensionCode`, `ClusterStem`, and both adapter submodules), the
      "future `who-fic-icd-api`" reference, the `ClusterStem`-parses
      claim (only `Cluster` parses), and the serde claim (clusters and
      errors have no serde); noted the crate's inline-tests-only layout
- [x] `specs/who-fic-icf.md` / `specs/who-fic-ichi.md`: serde scope
      corrected (enums/entries use plain derives, not canonical strings);
      ICHI's test list corrected (synthetic accept-list, always-`None`
      section tests — the old text claimed "real Beta-3 codes" and
      "section boundary tests" that never existed); documented the
      `Reader`/`Read` error-variant naming split and entry accessors
- [x] `specs/who-fic-linearization.md`: documented the real short-line
      defaults (`false`/`0`, not `None`, for non-`Option` fields),
      positional (name-unvalidated) column reading, first-char quote
      detection, CSV-unescaped `BrowserLink`, the inherent-`from_str`
      shape, the full `LinearizationError` variant list, and the
      `browser_link()`/`primary_tabulation()` accessors the spec had
      skipped
- [x] `specs/who-fic-claml.md`: documented the self-closing-only
      `SuperClass`/`SubClass` limitation (see backlog), unvalidated root
      element, optional `kind`, `ModifierClass` rubrics, trait-`FromStr`
      shape, non-streaming `from_reader`, `ClamlError` variants, and
      `quick-xml` being semver-public via `ClamlError::Xml`
- [x] `specs/who-fic-icd-api.md`: documented `Auth(String)` and the
      token-endpoint error carve-out, the unread `token_type`, the
      `Icd10Entity = Entity` alias, the `SearchResults`/
      `SearchResultEntity`/`LangString` shapes, builder method names, the
      no-features/required-serde stance, percent-encoding, and the
      mutex-held-across-refresh tradeoff
- [x] `AGENTS.md`: fixed the version claim ("currently 0.2.0/0.1.0" —
      two releases stale) to point at `Cargo.toml` instead of a number
- [x] `CHANGELOG.md`: scoped the 0.3.1 entry's "every crate gained a
      second runnable example" to the six crates it actually covered

## Phase 17 — Fix the three audit defects, republish (2026-08-06)

The three code-level defects from Phase 16's audit, fixed with regression
tests and shipped. Patch bumps: each fix makes behavior match what the
docs already promised, adding no new API surface.

- [x] `who-fic-icd-api/examples/search_and_traverse.rs`: extract the
      trailing numeric foundation ID from the search hit's URI before
      calling `client.entity()` (the call previously sent the full
      percent-encoded URI and 404'd)
- [x] serde feature forwarding: `who-fic-icd`'s `serde` feature now
      weakly forwards to `who-fic-claml?/serde` and
      `who-fic-linearization?/serde` (likewise `who-fic-icf`/
      `who-fic-ichi` for linearization), so `serde` +
      `linearization`/`claml` through the umbrella — or through the
      classification crates directly — makes `LinearizationRow`/
      `ClamlDocument` serializable, as `who-fic`'s README always claimed.
      Regression test: `who-fic/tests/serde_forwarding.rs` (new dev-deps
      on the two parser crates to name their types)
- [x] `who-fic-claml`: `SuperClass`/`SubClass` in start/end-tag pair form
      (`<SuperClass code="..."></SuperClass>`) are now recognized, not
      silently dropped; new unit test covering both elements in both
      forms
- [x] Specs updated in the same change (`who-fic.md`, `architecture.md`,
      `who-fic-claml.md` — gap language replaced with the fixed behavior
      and version boundary)
- [x] Full workspace verification (fmt, clippy, three test matrices, doc,
      feature powerset), `cargo semver-checks` against the published
      baselines, publish in dependency order, push, CI verified green

## Published (0.3.2 / 0.1.3 / 0.1.2)

- [x] `who-fic-claml` 0.1.3 — https://crates.io/crates/who-fic-claml
- [x] `who-fic-icd` 0.3.2 — https://crates.io/crates/who-fic-icd
- [x] `who-fic-icf` 0.3.2 — https://crates.io/crates/who-fic-icf
- [x] `who-fic-ichi` 0.3.2 — https://crates.io/crates/who-fic-ichi
- [x] `who-fic` 0.3.2 — https://crates.io/crates/who-fic
- [x] `who-fic-icd-api` 0.1.3 — https://crates.io/crates/who-fic-icd-api
- [x] `who-fic-linearization` stays 0.1.2 — unchanged this round

## Backlog / future subcrates (not scheduled)

- [ ] Semantic cluster validation for ICD-11 postcoordination (needs WHO data)
- [ ] Split `who-fic-icd` into `who-fic-icd-10` / `who-fic-icd-11` if the
      revisions grow enough to justify it

### Code follow-ups surfaced by the Phase 16 audit (not scheduled)

The three concrete defects the audit found were fixed and shipped in
Phase 17 (below). Still open, judgment-call improvements rather than
defects:

- [ ] Consider serde impls for `Cluster`/`ClusterStem` (canonical-string,
      like the code types) — currently the only value types in
      `who-fic-icd` without them
- [ ] Rustdoc examples for `Icd11ClassEntry`'s six accessors (the
      parallel `Icd10ClassEntry` accessors all have them)
