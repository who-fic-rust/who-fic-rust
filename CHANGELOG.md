# Changelog

All notable changes to the WHO-FIC-Rust crates are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The four crates (`who-fic`, `who-fic-icd`, `who-fic-icf`, `who-fic-ichi`)
version together (lockstep versioning).

## [Unreleased]

## [0.1.0] - 2026-08-03

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
