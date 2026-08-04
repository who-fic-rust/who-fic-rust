# who-fic-ichi

[![crates.io](https://img.shields.io/crates/v/who-fic-ichi.svg)](https://crates.io/crates/who-fic-ichi)
[![docs.rs](https://img.shields.io/docsrs/who-fic-ichi)](https://docs.rs/who-fic-ichi)
[![CI](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/who-fic-ichi.svg)](#license)

World Health Organization (WHO) [International Classification of Health
Interventions
(ICHI)](https://www.who.int/standards/classifications/international-classification-of-health-interventions)
code types for Rust.

Part of the [WHO-FIC-Rust](https://github.com/who-fic-rust/who-fic-rust)
workspace. See [`who-fic`](https://crates.io/crates/who-fic) for the
umbrella crate covering ICD, ICF, and ICHI together.

**Stability note:** ICHI's latest public release is a beta (Beta-3). This
crate's axis architecture (Target / Action / Means) is stable, but code
details may change before final adoption.

Every ICHI intervention code is built from three axes — Target, Action,
Means — combined as `TARGET.ACTION.MEANS`, e.g. `KAB.DB.AD`.

## Example

```rust
use who_fic_ichi::IchiCode;

let code: IchiCode = "kab.db.ad".parse().unwrap();
assert_eq!(code.to_string(), "KAB.DB.AD");
assert_eq!(code.target().as_str(), "KAB");
assert_eq!(code.action().as_str(), "DB");
assert_eq!(code.means().as_str(), "AD");
```

## Features

- `serde` — canonical-string `Serialize`/`Deserialize` for `IchiCode` and
  the axis types.
- `linearization` — `linearization` module: build an
  `IchiLinearizationIndex` (code → title lookup) from a
  [WHO linearization export](https://crates.io/crates/who-fic-linearization)
  you parse yourself. You supply the export file; this crate never bundles
  WHO content.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
