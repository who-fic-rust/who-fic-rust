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

## Published

- [x] `who-fic-icd` 0.1.0 — https://crates.io/crates/who-fic-icd
- [x] `who-fic-icf` 0.1.0 — https://crates.io/crates/who-fic-icf
- [x] `who-fic-ichi` 0.1.0 — https://crates.io/crates/who-fic-ichi
- [x] `who-fic` 0.1.0 — https://crates.io/crates/who-fic

## Backlog / future subcrates (not scheduled)

- [ ] `who-fic-icd-api`: WHO ICD-API client (OAuth2 client-credentials,
      entity lookup, search)
- [ ] Data loaders for user-supplied official WHO exports
      (`who-fic-*-data` crates)
- [ ] Semantic cluster validation for ICD-11 postcoordination (needs WHO data)
- [ ] Split `who-fic-icd` into `who-fic-icd-10` / `who-fic-icd-11` if the
      revisions grow enough to justify it
