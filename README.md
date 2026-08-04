# WHO-FIC-Rust

[![CI](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Rust crates for the World Health Organization (WHO) Family of
International Classifications (FIC):

- International Classification of Diseases (ICD)
- International Classification of Functioning, Disability and Health (ICF)
- International Classification of Health Interventions (ICHI)

## Crates

| Crate | | Description |
|---|---|---|
| [`who-fic`](who-fic) | [![crates.io](https://img.shields.io/crates/v/who-fic.svg)](https://crates.io/crates/who-fic) | Umbrella crate: ICD + ICF + ICHI in one dependency |
| [`who-fic-icd`](who-fic-icd) | [![crates.io](https://img.shields.io/crates/v/who-fic-icd.svg)](https://crates.io/crates/who-fic-icd) | ICD-10 and ICD-11 code types |
| [`who-fic-icf`](who-fic-icf) | [![crates.io](https://img.shields.io/crates/v/who-fic-icf.svg)](https://crates.io/crates/who-fic-icf) | ICF code types |
| [`who-fic-ichi`](who-fic-ichi) | [![crates.io](https://img.shields.io/crates/v/who-fic-ichi.svg)](https://crates.io/crates/who-fic-ichi) | ICHI code types |
| [`who-fic-linearization`](who-fic-linearization) | [![crates.io](https://img.shields.io/crates/v/who-fic-linearization.svg)](https://crates.io/crates/who-fic-linearization) | Parser for WHO's shared TSV export format (ICD-11/ICF/ICHI) |
| [`who-fic-claml`](who-fic-claml) | [![crates.io](https://img.shields.io/crates/v/who-fic-claml.svg)](https://crates.io/crates/who-fic-claml) | Parser for ClaML (ISO 13120) XML, used by ICD-10 |
| [`who-fic-icd-api`](who-fic-icd-api) | [![crates.io](https://img.shields.io/crates/v/who-fic-icd-api.svg)](https://crates.io/crates/who-fic-icd-api) | Async client for the live WHO ICD-API |

Each crate implements code **syntax, structure, and validation** — parsing,
formatting, hierarchy navigation, and (where applicable) an optional loader
for a WHO export file you supply yourself. WHO's classification *content*
(titles, definitions, the full tabular lists) is copyrighted by WHO and
never vendored by these crates — see
[specs/architecture.md](specs/architecture.md) for the full rationale.

## Documentation

- **New here? Start with [TUTORIAL.md](TUTORIAL.md)** — a guided
  walkthrough of all seven crates, code parsing through the live API
  client.
- [FAQ.md](FAQ.md) — specific questions ("does this validate a code
  exists?", "is this crate async?", "will a future release break my
  code?") that don't fit a walkthrough
- [plan.md](plan.md) — project goals, design principles, architecture
- [tasks.md](tasks.md) — task breakdown and what's shipped
- [specs/](specs/index.md) — detailed per-crate specifications (the
  single source of truth for documented behavior)
- [CHANGELOG.md](CHANGELOG.md)
- [CONTRIBUTING.md](CONTRIBUTING.md) — for contributors
- [AGENTS.md](AGENTS.md) (or [CLAUDE.md](CLAUDE.md)) — for AI coding
  agents working in this repository

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
