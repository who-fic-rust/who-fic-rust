# who-fic-linearization

[![crates.io](https://img.shields.io/crates/v/who-fic-linearization.svg)](https://crates.io/crates/who-fic-linearization)
[![docs.rs](https://img.shields.io/docsrs/who-fic-linearization)](https://docs.rs/who-fic-linearization)
[![CI](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/who-fic-rust/who-fic-rust/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/who-fic-linearization.svg)](#license)

A parser for WHO's **Simplified Linearization Output** format: the
tab-separated export WHO's ICD-11 Maintenance Platform
(`icd.who.int/dev11/downloads`) produces for ICD-11 (MMS), ICF, and ICHI.
All three classifications export from the same platform in the same base
tabular shape, which is why this is one general-purpose crate rather than
three copies of the same parser.

Part of the [WHO-FIC-Rust](https://github.com/who-fic-rust/who-fic-rust)
workspace. This crate is **format-only** — it has no knowledge of ICD,
ICF, or ICHI code syntax. See
[`who-fic-icd`](https://crates.io/crates/who-fic-icd)'s `icd11::linearization`
module, [`who-fic-icf`](https://crates.io/crates/who-fic-icf)'s
`linearization` module, and
[`who-fic-ichi`](https://crates.io/crates/who-fic-ichi)'s `linearization`
module for the classification-specific adapters built on top of this crate.

This crate parses a file the *user* supplies (downloaded from
`icd.who.int/dev11/downloads` under WHO's own terms) — it does not fetch,
bundle, or embed WHO's classification content.

## Install

```toml
[dependencies]
who-fic-linearization = "0.1"
```

## Example

```rust
use who_fic_linearization::LinearizationReader;

let tsv = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
           http://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"link\"\tTrue\t0\n";

for result in LinearizationReader::from_str(tsv) {
    let row = result.unwrap();
    assert_eq!(row.code(), Some("1A00"));
    assert_eq!(row.title(), "Cholera"); // depth-dash markers stripped
}
```

More runnable examples in [`examples/`](examples): `stream_and_filter.rs`
streams a multi-row export, tells chapter/block/category rows apart, and
handles a malformed row without aborting the rest of the stream.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
