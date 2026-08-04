# who-fic-icf

[![crates.io](https://img.shields.io/crates/v/who-fic-icf.svg)](https://crates.io/crates/who-fic-icf)
[![docs.rs](https://img.shields.io/docsrs/who-fic-icf)](https://docs.rs/who-fic-icf)
[![CI](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/who-fic-icf.svg)](#license)

World Health Organization (WHO) [International Classification of
Functioning, Disability and Health
(ICF)](https://www.who.int/standards/classifications/international-classification-of-functioning-disability-and-health)
code types for Rust.

Part of the [WHO-FIC-Rust](https://github.com/who-fic-rust/who-fic-rust)
workspace. See [`who-fic`](https://crates.io/crates/who-fic) for the
umbrella crate covering ICD, ICF, and ICHI together.

ICF codes have three parts: a component letter (`Component`), a numeric
hierarchy (`IcfCode`), and optional qualifiers (`QualifiedIcfCode`). This
crate makes the ICF's qualifier structure — which differs per component,
including the environmental barrier vs. facilitator distinction — a
compile-time-enforced part of the type system.

## Install

```toml
[dependencies]
who-fic-icf = "0.3"
```

## Example

```rust
use std::str::FromStr;
use who_fic_icf::{Component, IcfCode, QualifiedIcfCode};

// The bare hierarchy code, as it appears in the tabulation.
let code = IcfCode::from_str("b280").unwrap();
assert_eq!(code.component(), Component::BodyFunctions);
assert_eq!(code.parent().unwrap().as_str(), "b2");

// The same code with an assessment qualifier attached.
let qualified = QualifiedIcfCode::from_str("b280.2").unwrap();
assert_eq!(qualified.code(), &code);

// Environmental factors distinguish barriers (".") from facilitators ("+").
assert!(QualifiedIcfCode::from_str("e150.2").is_ok()); // barrier
assert!(QualifiedIcfCode::from_str("e150+2").is_ok()); // facilitator
assert!(QualifiedIcfCode::from_str("b280+2").is_err()); // facilitator marker invalid outside 'e'
```

More runnable examples in [`examples/`](examples): `hierarchy_walk.rs`
walks a code all the way up to its chapter, one level at a time.

## Features

- `serde` — canonical-string `Serialize`/`Deserialize` for `IcfCode` and
  `QualifiedIcfCode`.
- `linearization` — `linearization` module: build an
  `IcfLinearizationIndex` (code → title lookup) from a
  [WHO linearization export](https://crates.io/crates/who-fic-linearization)
  you parse yourself. You supply the export file; this crate never bundles
  WHO content.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
