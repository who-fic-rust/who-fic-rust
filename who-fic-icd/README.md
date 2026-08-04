# who-fic-icd

[![crates.io](https://img.shields.io/crates/v/who-fic-icd.svg)](https://crates.io/crates/who-fic-icd)
[![docs.rs](https://img.shields.io/docsrs/who-fic-icd)](https://docs.rs/who-fic-icd)
[![CI](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/who-fic-icd.svg)](#license)

World Health Organization (WHO) [International Classification of Diseases
(ICD)](https://www.who.int/standards/classifications/classification-of-diseases)
— ICD-10 and ICD-11 code types for Rust.

Part of the [WHO-FIC-Rust](https://github.com/who-fic-rust/who-fic-rust)
workspace. See [`who-fic`](https://crates.io/crates/who-fic) for the
umbrella crate covering ICD, ICF, and ICHI together.

This crate implements code **syntax and structure** — parsing, validation,
formatting, chapter lookup, ICD-11 extension codes and postcoordination
cluster syntax. It does not vendor WHO's classification content (titles,
descriptions); see the crate's rustdoc for details.

## Install

```toml
[dependencies]
who-fic-icd = "0.3"
```

## Example

```rust
use std::str::FromStr;
use who_fic_icd::icd10::Icd10Code;
use who_fic_icd::icd11::Icd11Code;

let code = Icd10Code::from_str("I63.9").unwrap();
assert_eq!(code.category(), "I63");
assert_eq!(code.subdivision(), Some("9"));

let code = Icd11Code::from_str("8B20").unwrap();
assert!(code.chapter().is_some());
```

More runnable examples in [`examples/`](examples): `postcoordination_cluster.rs`
walks ICD-11 cluster syntax (`&`/`/`) in more depth.

## Features

- `serde` — canonical-string `Serialize`/`Deserialize` for all code types.
- `claml` — `icd10::claml` module: build an `Icd10ClamlIndex` (code → title
  lookup) from a [ClaML](https://crates.io/crates/who-fic-claml) document
  you parse yourself. You supply the ICD-10 export file; this crate never
  bundles WHO content.
- `linearization` — `icd11::linearization` module: build an
  `Icd11LinearizationIndex` (code → title/chapter/grouping lookup) from a
  [WHO linearization export](https://crates.io/crates/who-fic-linearization)
  you parse yourself, same principle.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
