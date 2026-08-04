# Changelog

All notable changes to the WHO-FIC-Rust crates are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The four original crates (`who-fic`, `who-fic-icd`, `who-fic-icf`,
`who-fic-ichi`) version together (lockstep versioning).
`who-fic-linearization`, `who-fic-claml`, and `who-fic-icd-api` are newer
crates with their own independent version.

## [Unreleased]

## who-fic-icd-api [0.1.0] - 2026-08-04

Initial release: an async client for the live WHO ICD-API (`id.who.int`),
the one crate in this workspace that makes network calls and requires
user-supplied credentials. Endpoints and the OAuth2 flow verified directly
against WHO's own OpenAPI spec.

### Added

- `IcdApiClient` / `IcdApiClientBuilder`: OAuth2 client-credentials
  authentication with automatic token caching and refresh; configurable
  token/API base URLs for testing against a local mock server.
- `entity` / `entity_search` (ICD-11 Foundation), `linearization_entity` /
  `search` (a specific linearization such as MMS), `code_info` /
  `code_info_typed` (resolve a code to its entity and postcoordination
  axis breakdown), `icd10_category` / `icd10_category_typed`.
- `Entity`, `CodeInfo`, `SearchResults`, `Icd10Entity` response types with
  permissive deserialization.
- `IcdApiError` (`#[non_exhaustive]`).

## who-fic / who-fic-icd / who-fic-icf / who-fic-ichi [0.2.0] - 2026-08-03

### Added

- `who-fic-icd`: optional `claml` feature (`icd10::claml` module,
  `Icd10ClamlIndex`) and optional `linearization` feature
  (`icd11::linearization` module, `Icd11LinearizationIndex`) — build a
  code → title lookup from a user-supplied WHO export, via the new
  `who-fic-claml` / `who-fic-linearization` crates.
- `who-fic-icf`, `who-fic-ichi`: optional `linearization` feature
  (`linearization` module, `IcfLinearizationIndex` /
  `IchiLinearizationIndex`), same principle.
- `who-fic`: `claml` and `linearization` features forwarding to the above.

## who-fic-linearization / who-fic-claml [0.1.0] - 2026-08-03

Initial release of two new general-purpose format-parser crates, split out
because ICD-11 (MMS), ICF, and ICHI all export from WHO's platform in one
shared tab-separated "Simplified Linearization Output" shape, while ICD-10
uses the separate ClaML XML format.

### Added

- `who-fic-linearization`: `LinearizationRow` / `LinearizationReader` for
  WHO's Simplified Linearization Output TSV format.
- `who-fic-claml`: `ClamlDocument` / `Class` / `Rubric` / `Label` /
  `ModifierClass` / `Modifier` for ClaML (ISO 13120) XML.
- Optional `serde` feature on both.

## who-fic / who-fic-icd / who-fic-icf / who-fic-ichi [0.1.0] - 2026-08-03

Initial release.

### Added

- `who-fic-icd`: `Icd10Code` and `Icd11Code` types with parsing,
  validation, formatting, chapter lookup, and ICD-11 extension codes and
  postcoordination cluster syntax.
- `who-fic-icf`: `Component`, `IcfCode` (with hierarchy level and parent
  navigation), `Qualifier`, and component-aware `QualifiedIcfCode`
  (including the ICF environmental barrier/facilitator distinction).
- `who-fic-ichi`: `Target`/`Action`/`Means` axis types, composed
  `IchiCode`, and `Section` classification.
- `who-fic`: umbrella crate re-exporting each classification behind a
  same-named feature (`icd`, `icf`, `ichi`, all default), plus the shared
  `Classification` enum and `FicError` type.
- Optional `serde` feature on every crate (codes serialize as their
  canonical string).
