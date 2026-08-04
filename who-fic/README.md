# who-fic

[![crates.io](https://img.shields.io/crates/v/who-fic.svg)](https://crates.io/crates/who-fic)
[![docs.rs](https://img.shields.io/docsrs/who-fic)](https://docs.rs/who-fic)
[![CI](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/who-fic.svg)](#license)

World Health Organization (WHO) Family of International Classifications
(FIC) for Rust: [ICD](https://crates.io/crates/who-fic-icd),
[ICF](https://crates.io/crates/who-fic-icf), and
[ICHI](https://crates.io/crates/who-fic-ichi) in one crate.

Part of the [WHO-FIC-Rust](https://github.com/who-fic-rust/who-fic-rust)
workspace. Each classification is available directly as its own crate too
— use `who-fic` if you want more than one, or want the shared
`Classification`/`FicError` vocabulary.

## Usage

```toml
[dependencies]
who-fic = "0.3"
```

Each classification is behind a same-named feature, all enabled by
default (`icd`, `icf`, `ichi`) — disable defaults and pick only what you
need to trim dependencies:

```toml
[dependencies]
who-fic = { version = "0.3", default-features = false, features = ["icd"] }
```

## Example

```rust
use std::str::FromStr;

let code = who_fic::icd::icd11::Icd11Code::from_str("8B20").unwrap();
assert_eq!(code.as_str(), "8B20");

let code = who_fic::icf::IcfCode::from_str("b280").unwrap();
assert_eq!(code.component(), who_fic::icf::Component::BodyFunctions);

let code = who_fic::ichi::IchiCode::from_str("KAB.DB.AD").unwrap();
assert_eq!(code.target().as_str(), "KAB");
```

## What this crate does not do

WHO owns the copyright on classification *content* (code titles,
descriptions, the full tabular lists). These crates implement code
**syntax and structure** only — parsing, validation, formatting, and
navigation — not existence checks against WHO's official data. See
`specs/architecture.md` in the repository for the full rationale.

## Features

- `icd`, `icf`, `ichi` — pull in and re-export the matching subcrate
  (all default).
- `serde` — forwards to each enabled subcrate's `serde` feature.
- `claml` — forwards to `who-fic-icd`'s `claml` feature (ICD-10 data
  loading from a user-supplied [ClaML](https://crates.io/crates/who-fic-claml)
  export).
- `linearization` — forwards to `who-fic-icd`/`who-fic-icf`/`who-fic-ichi`'s
  `linearization` feature (ICD-11/ICF/ICHI data loading from a
  user-supplied [WHO linearization export](https://crates.io/crates/who-fic-linearization)).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
