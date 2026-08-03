# Plan: WHO-FIC-Rust

Rust workspace implementing the World Health Organization (WHO) Family of
International Classifications (FIC) as strongly-typed, well-tested crates.

## Goal

Publishable Rust crates that let health-informatics developers parse,
validate, format, and navigate WHO-FIC classification codes:

- `who-fic` — umbrella crate: shared traits, shared error types, and
  feature-gated re-exports of the subcrates.
- `who-fic-icd` — International Classification of Diseases (ICD-10 and ICD-11).
- `who-fic-icf` — International Classification of Functioning, Disability and
  Health (ICF).
- `who-fic-ichi` — International Classification of Health Interventions (ICHI).

Further subcrates are created as needed using the parent crate's name as a
prefix (e.g. `who-fic-icd-api`); see "Future subcrates" below.

## Background

WHO-FIC is a suite of classifications for recording health information in a
standardized, internationally comparable way. The three reference
classifications are:

| Classification | Answers | Example code |
|---|---|---|
| ICD | What diseases/conditions? | `8B20` (ICD-11), `I63.9` (ICD-10) |
| ICF | What functioning/disability? | `b280.2` |
| ICHI | What health interventions? | `KAB.DB.AD` |

Each classification has its own code syntax, hierarchy, and semantics, which
is why each gets its own crate. What they share — the notion of a code that
can be parsed, validated, displayed, and situated in a hierarchy — lives in
`who-fic`.

## Design principles

1. **Type safety first.** Each classification defines its own code types
   (newtypes/structs, not bare `String`s). Invalid codes are unrepresentable
   after construction: "parse, don't validate."
2. **Structure without data.** WHO classification *content* (titles,
   definitions, the full entity catalogs) is copyrighted by WHO and licensed
   separately. The crates implement code **syntax, structure, and semantics**
   (formats, axes, qualifiers, chapter ranges) and provide loaders/APIs so
   users can bring their own licensed data. We do not vendor WHO content.
3. **Standard Rust conventions.** `FromStr`/`Display`/`TryFrom` for codes,
   `std::error::Error` for errors, `#[non_exhaustive]` where WHO may extend,
   optional `serde` behind a feature flag on every crate.
4. **Minimal dependency footprint.** Core crates aim for zero required
   dependencies; `serde` is optional. Heavier concerns (HTTP clients for the
   WHO ICD-API, file-format loaders) go into future subcrates so the core
   stays lean.
5. **Umbrella with features.** `who-fic` re-exports each subcrate behind a
   same-named feature (`icd`, `icf`, `ichi`), all enabled by default, so users
   can depend on one crate and trim what they don't need.

## Architecture

Cargo workspace at the repository root:

```
who-fic-rust/                  (workspace root, virtual manifest)
├── Cargo.toml                 [workspace] members, shared package metadata
├── plan.md  tasks.md          this plan and its task breakdown
├── specs/                     detailed specifications (specs/index.md is the map)
├── who-fic/                   umbrella crate
│   └── src/lib.rs             common traits + feature-gated re-exports
├── who-fic-icd/               ICD crate (modules: icd10, icd11)
├── who-fic-icf/               ICF crate (components b/s/d/e, qualifiers)
└── who-fic-ichi/              ICHI crate (Target–Action–Means axis codes)
```

Dependency graph (arrows = "depends on"):

```
who-fic ──(feature icd)──▶ who-fic-icd
        ──(feature icf)──▶ who-fic-icf
        ──(feature ichi)─▶ who-fic-ichi
```

The subcrates are independent of each other and of `who-fic` (no cycles;
shared traits are defined in `who-fic` and *implemented* there for the
subcrates' types via the re-export layer, or the subcrates stay trait-free
and `who-fic` provides blanket integration — see `specs/who-fic.md`).

## Phases

### Phase 0 — Workspace scaffolding
Workspace `Cargo.toml`, the four crates with placeholder `lib.rs`, shared
`[workspace.package]` metadata, `.gitignore`, `cargo build` green.

### Phase 1 — Common core (`who-fic`)
`FicError` (shared parse-error type), `Classification` enum, code-behavior
conventions, feature wiring (`icd`/`icf`/`ichi`/`serde`), crate-level docs.

### Phase 2 — ICD (`who-fic-icd`)
ICD-10 and ICD-11 code types: parse/validate/format, chapter enums and
chapter-of-code lookup, ICD-11 stem-code vs. extension-code distinction,
groundwork for ICD-11 postcoordination clusters (`/`, `&`).

### Phase 3 — ICF (`who-fic-icf`)
Component enum (`b`/`s`/`d`/`e`), hierarchical code levels (chapter through
fourth level), qualifier parsing per component (including environmental
barrier `.n` vs. facilitator `+n`), generic qualifier scale.

### Phase 4 — ICHI (`who-fic-ichi`)
Axis types Target/Action/Means, full `XXX.AA.MM` intervention codes,
section classification, extension-code groundwork.

### Phase 5 — Integration & polish
`who-fic` re-export surface verified under every feature combination
(`cargo hack` or explicit matrix), `serde` round-trip tests everywhere,
property tests for parsers, rustdoc examples on all public items.

### Phase 6 — Release readiness
CI (fmt, clippy, test, feature matrix, MSRV check), CHANGELOG, crate
metadata (description, keywords, categories), README per crate,
`cargo publish --dry-run` for all four crates in dependency order.

## Future subcrates (create when needed, not up front)

- `who-fic-icd-api` — client for the WHO ICD-API (`id.who.int`, OAuth2
  client-credentials), async, `reqwest`-based.
- `who-fic-icd-10` / `who-fic-icd-11` — split out only if the revisions grow
  enough (data loaders, per-revision tooling) to justify separate crates;
  until then they are modules `icd10`/`icd11` inside `who-fic-icd`.
- `who-fic-icf-data`, `who-fic-ichi-data` — loaders for user-supplied
  official WHO data exports (licensing stays the user's responsibility).

## Risks and open questions

- **WHO licensing.** Redistribution of classification content requires WHO
  permission; mitigated by principle 2 (structure without data).
- **ICHI stability.** ICHI's latest public release is a beta; its code
  structure is stable but details may shift. Mark ICHI types
  `#[non_exhaustive]` where practical and document the beta status.
- **ICD-11 postcoordination** is the most complex syntax in the family
  (clusters combining stem and extension codes). Phase 2 delivers stem and
  extension codes plus cluster *parsing*; full semantic validation of
  clusters (which extensions are allowed on which stems) requires WHO data
  and is deferred to a data-aware subcrate.
- **Validation depth.** Syntactic validation (format) is decidable without
  WHO data; existence validation (is `1A95.3` an assigned code?) is not.
  The core crates do syntax; existence checks belong to data/API subcrates.

## Definition of done

- `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt
  --check`, and `cargo doc` all pass for every crate and feature combination.
- Every public type has rustdoc with at least one tested example.
- The specs in `specs/` match the implemented behavior.
- `tasks.md` fully checked off.
