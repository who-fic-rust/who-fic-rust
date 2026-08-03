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

- [ ] `FicError` parse-error type (`std::error::Error + Display`), covering
      empty input, bad length, bad character, bad structure
- [ ] `Classification` enum (`Icd10`, `Icd11`, `Icf`, `Ichi`),
      `#[non_exhaustive]`
- [ ] Document the code-type conventions every subcrate follows
      (`FromStr`, `Display`, `TryFrom<&str>`, `as_str`, ordering, hashing)
- [ ] `serde` feature that forwards to each enabled subcrate's `serde` feature
- [ ] Crate-level rustdoc with a worked example per classification
- [ ] Unit tests: feature-gated re-exports resolve (`who_fic::icd::...`, etc.)

## Phase 2 — ICD (`who-fic-icd`)

See [specs/who-fic-icd.md](specs/who-fic-icd.md).

- [ ] Module `icd10`: `Icd10Code` — parse/validate `A00`–`Z99` categories with
      optional subdivision (`I63.9`), `category()` / `subdivision()` accessors
- [ ] Module `icd10`: `Icd10Chapter` enum (I–XXII) and code→chapter mapping
- [ ] Module `icd11`: `Icd11Code` — 4-character stem codes with optional
      subdivisions (`8B20`, `CA40.0`), excluded letters `I`/`O` enforced
- [ ] Module `icd11`: extension codes (`X…` prefix) as a distinct type or flag
- [ ] Module `icd11`: `Icd11Chapter` enum (01–26, V, X) and code→chapter
      mapping from the leading character
- [ ] Module `icd11`: residual-category detection (`.Y` other-specified,
      `.Z` unspecified)
- [ ] Module `icd11`: postcoordination `Cluster` parsing (`/` for multiple
      stems, `&` for stem+extension) — syntax only
- [ ] `serde` feature: transparent string serialization, round-trip tests
- [ ] Property tests: parse-format round trip; rejection of malformed input
- [ ] Rustdoc examples on all public items

## Phase 3 — ICF (`who-fic-icf`)

See [specs/who-fic-icf.md](specs/who-fic-icf.md).

- [ ] `Component` enum: BodyFunctions (`b`), BodyStructures (`s`),
      ActivitiesAndParticipation (`d`), EnvironmentalFactors (`e`)
- [ ] `IcfCode` — component letter + 1–5 digits; `level()` accessor
      (Chapter / Second / Third / Fourth) and `parent()` navigation
- [ ] `Qualifier` — generic scale 0–4, 8 (not specified), 9 (not applicable)
- [ ] Qualified codes: component-specific qualifier counts and meanings
      (b: 1; s: up to 3; d: performance + capacity; e: barrier `.n` or
      facilitator `+n`)
- [ ] `serde` feature + round-trip tests
- [ ] Property tests and rustdoc examples

## Phase 4 — ICHI (`who-fic-ichi`)

See [specs/who-fic-ichi.md](specs/who-fic-ichi.md).

- [ ] Axis types: `Target` (3 chars), `Action` (2 chars), `Means` (2 chars)
- [ ] `IchiCode` — `TTT.AA.MM` parse/format, accessors per axis
- [ ] `Section` enum (body systems & functions / activities & participation /
      environment) derived from the target
- [ ] Extension-code groundwork (documented, type stubbed or deferred)
- [ ] `serde` feature + round-trip tests
- [ ] Property tests and rustdoc examples
- [ ] Document ICHI beta status in crate docs

## Phase 5 — Integration & polish

- [ ] Feature-combination build matrix (e.g. `cargo hack --feature-powerset`)
      for `who-fic`
- [ ] `serde` round-trip integration tests through the umbrella crate
- [ ] Doc tests pass for every crate; intra-doc links resolve
- [ ] Verify specs/*.md match implemented behavior; update where they diverge

## Phase 6 — Release readiness

- [ ] CI workflow: fmt, clippy `-D warnings`, test, doc, feature matrix, MSRV
- [ ] Decide and pin MSRV (`rust-version` in `[workspace.package]`)
- [ ] Per-crate README + crates.io metadata (description, keywords,
      categories, docs.rs config)
- [ ] CHANGELOG.md
- [ ] `cargo publish --dry-run` in dependency order (subcrates, then umbrella)

## Backlog / future subcrates (not scheduled)

- [ ] `who-fic-icd-api`: WHO ICD-API client (OAuth2 client-credentials,
      entity lookup, search)
- [ ] Data loaders for user-supplied official WHO exports
      (`who-fic-*-data` crates)
- [ ] Semantic cluster validation for ICD-11 postcoordination (needs WHO data)
- [ ] Split `who-fic-icd` into `who-fic-icd-10` / `who-fic-icd-11` if the
      revisions grow enough to justify it
