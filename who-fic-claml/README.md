# who-fic-claml

[![crates.io](https://img.shields.io/crates/v/who-fic-claml.svg)](https://crates.io/crates/who-fic-claml)
[![docs.rs](https://img.shields.io/docsrs/who-fic-claml)](https://docs.rs/who-fic-claml)
[![CI](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/who-fic-claml.svg)](#license)

A parser for **ClaML** (Classification Markup Language), the XML format
standardized as ISO 13120 for exchanging healthcare classification
systems. WHO distributes ICD-10 in ClaML.

Part of the [WHO-FIC-Rust](https://github.com/who-fic-rust/who-fic-rust)
workspace. This crate is **format-only** — it has no ICD-10-specific
knowledge. See [`who-fic-icd`](https://crates.io/crates/who-fic-icd)'s
`icd10::claml` module for the ICD-10-specific adapter built on top of this
crate.

This crate parses XML the *user* supplies (obtained under WHO's or the
relevant maintainer's own terms) — it does not fetch, bundle, or embed
classification content.

## Install

```toml
[dependencies]
who-fic-claml = "0.1"
```

## Example

```rust
use who_fic_claml::ClamlDocument;
use std::str::FromStr;

let xml = r#"
<ClaML version="2.0">
  <Class code="A00" kind="category">
    <SuperClass code="A00-A09"/>
    <Rubric kind="preferred">
      <Label xml:lang="en">Cholera</Label>
    </Rubric>
  </Class>
</ClaML>
"#;

let doc = ClamlDocument::from_str(xml).unwrap();
let class = &doc.classes()[0];
assert_eq!(class.code(), "A00");
assert_eq!(class.preferred_label("en"), Some("Cholera"));
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
