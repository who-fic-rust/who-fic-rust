# Specifications: WHO-FIC-Rust

Detailed specifications for the WHO Family of International Classifications
(FIC) Rust workspace. Read [plan.md](../plan.md) first for the overall plan
and [tasks.md](../tasks.md) for the task breakdown.

**These files are the single source of truth for documented behavior.**
Implementation changes and spec changes land together — see
[AGENTS.md](../AGENTS.md) (or [CLAUDE.md](../CLAUDE.md), which imports it)
for the workflow this repository follows and for operational guides
([AGENTS/release.md](../AGENTS/release.md),
[AGENTS/testing.md](../AGENTS/testing.md),
[AGENTS/lessons.md](../AGENTS/lessons.md)) that sit alongside these specs
without duplicating them. New to the crates as a *user* rather than a
contributor? Start with [TUTORIAL.md](../TUTORIAL.md) instead.

## Spec map

- [architecture.md](architecture.md) — workspace layout, crate conventions,
  features, errors, testing, CI, licensing constraints
- [who-fic.md](who-fic.md) — umbrella crate: shared types and feature-gated
  re-exports
- [who-fic-icd.md](who-fic-icd.md) — International Classification of
  Diseases (ICD-10 and ICD-11)
- [who-fic-icf.md](who-fic-icf.md) — International Classification of
  Functioning, Disability and Health (ICF)
- [who-fic-ichi.md](who-fic-ichi.md) — International Classification of
  Health Interventions (ICHI)
- [who-fic-linearization.md](who-fic-linearization.md) — parser for WHO's
  shared "Simplified Linearization Output" TSV format (ICD-11 MMS, ICF,
  ICHI exports)
- [who-fic-claml.md](who-fic-claml.md) — parser for ClaML (ISO 13120), the
  XML format WHO distributes ICD-10 in
- [who-fic-icd-api.md](who-fic-icd-api.md) — async client for the live WHO
  ICD-API (`id.who.int`)

## Relationship to the seed specs

The original one-page seed specs remain at `spec/index.md` (root) and
`<crate>/spec/index.md`. The files in this `specs/` directory supersede them;
the seed specs are kept as the original statement of intent.

## Normative sources

- WHO-FIC overview: https://www.who.int/standards/classifications
- ICD: https://www.who.int/standards/classifications/classification-of-diseases
- ICF: https://www.who.int/standards/classifications/international-classification-of-functioning-disability-and-health
- ICHI: https://www.who.int/standards/classifications/international-classification-of-health-interventions
- ICD-11 browser & reference guide: https://icd.who.int/
- WHO ICD-API: https://icd.who.int/icdapi

Where this spec and the official WHO reference guides disagree, the WHO
guides win; file an issue and fix the spec.
