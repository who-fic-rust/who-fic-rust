# who-fic-icd

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

## Features

- `serde` — canonical-string `Serialize`/`Deserialize` for all code types.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
