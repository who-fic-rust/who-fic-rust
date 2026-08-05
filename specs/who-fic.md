# Spec: `who-fic` (umbrella crate)

The entry-point crate. Users who want "WHO-FIC support" depend on this one
crate; users who want a single classification may depend on a subcrate
directly. `who-fic` contains the small amount of genuinely shared vocabulary
plus feature-gated re-exports.

## Features

| Feature | Default | Effect |
|---|---|---|
| `icd` | yes | depends on and re-exports `who-fic-icd` as `who_fic::icd` |
| `icf` | yes | depends on and re-exports `who-fic-icf` as `who_fic::icf` |
| `ichi` | yes | depends on and re-exports `who-fic-ichi` as `who_fic::ichi` |
| `serde` | no | forwards to each enabled *classification* subcrate's `serde` feature (see caveat below) |
| `claml` | no | forwards to `who-fic-icd`'s `claml` feature (ICD-10 data loading) |
| `linearization` | no | forwards to `who-fic-icd`/`who-fic-icf`/`who-fic-ichi`'s `linearization` feature (ICD-11/ICF/ICHI data loading) |

```toml
[features]
default = ["icd", "icf", "ichi"]
icd = ["dep:who-fic-icd"]
icf = ["dep:who-fic-icf"]
ichi = ["dep:who-fic-ichi"]
serde = ["who-fic-icd?/serde", "who-fic-icf?/serde", "who-fic-ichi?/serde"]
claml = ["who-fic-icd?/claml"]
linearization = ["who-fic-icd?/linearization", "who-fic-icf?/linearization", "who-fic-ichi?/linearization"]
```

The crate must compile with `--no-default-features` (it is then just the
common module).

Caveat on `serde` + data loading (known gap, tracked in tasks.md's
backlog): as the TOML above shows, `serde` forwards only to the three
classification crates. It does not reach `who-fic-linearization` or
`who-fic-claml`, so with `features = ["serde", "linearization"]` (or
`"claml"`) the parser crates' own types (`LinearizationRow`,
`ClamlDocument`, …) reached through the adapter modules are *not*
serializable. Depend on the parser crate directly with its `serde`
feature if you need that. `who-fic`'s own `Classification` and `FicError`
have no serde impls under any feature.

## Re-export surface

```rust
#[cfg(feature = "icd")]
pub use who_fic_icd as icd;
#[cfg(feature = "icf")]
pub use who_fic_icf as icf;
#[cfg(feature = "ichi")]
pub use who_fic_ichi as ichi;
```

Whole-crate re-exports (not item-by-item): the subcrates own their public
API; `who-fic` does not curate or rename it.

## Common module

### `Classification`

Identifies a member of the family, for use in APIs that handle codes from
multiple classifications (routing, tagging, diagnostics):

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Classification {
    Icd10,
    Icd11,
    Icf,
    Ichi,
}
```

- `as_str(&self) -> &'static str` and `Display` print the conventional
  short name: `"ICD-10"`, `"ICD-11"`, `"ICF"`, `"ICHI"`.
- `FromStr` accepts the short names case-insensitively and ignores *all*
  hyphens, wherever they appear (`"icd11"`, `"ICD-11"`, and degenerate
  forms like `"I-C-F"` all parse). Its error type is
  `ParseClassificationError`, a single-field struct (`Debug + Clone +
  PartialEq + Eq + Display + std::error::Error`; not `#[non_exhaustive]`,
  and the offending input is reported via `Display`, not an accessor).
- `#[non_exhaustive]` because WHO-FIC contains related and derived
  classifications that may be added later.

### `FicError`

The shared parse-error shape:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FicError {
    Empty,
    InvalidLength { expected: &'static str, found: usize },
    InvalidCharacter { position: usize, found: char },
    InvalidStructure { reason: &'static str },
}
```

Implements `Display` + `std::error::Error` (messages: `"empty input"`,
`"invalid length: expected {expected}, found {found}"`, `"invalid
character {found:?} at position {position}"`, `"invalid structure:
{reason}"`). Subcrates define error types of their own with the same
four-variant *shape* (to remain independent of the umbrella), but not
field-for-field identical: the two ICD errors' `InvalidLength` has no
`expected` field (the `From` impl synthesizes one), and every
`IchiParseError` variant carries an extra `axis: Option<Axis>` (the
`From` impl drops it). `who-fic` provides `From` conversions to
`FicError` for the four *parse* errors — `Icd10ParseError`,
`Icd11ParseError`, `IcfParseError`, `IchiParseError` — gated on the
matching feature, so multi-classification callers can unify on one error
type. The four data-loading `*Index` error types deliberately have no
`From<…> for FicError`: they are I/O-ish loader errors, not code parse
errors.

### `AnyCode` (deliberately deferred)

A `who_fic::AnyCode` enum wrapping any classification's code type is
plausible but not part of the initial scope; add it only when a concrete
consumer needs it. Recorded here so the decision is visible.

## Documentation

Crate-level rustdoc shows one parse-and-inspect example (ICD-11, gated
`cfg(feature = "icd")` so docs build under any feature set). The
per-classification crates carry their own worked examples; repeating one
per classification here was planned but not done — the READMEs and
`TUTORIAL.md` cover that ground instead.

## Tests

- Compile-and-use smoke test per feature: `who_fic::icd`, `who_fic::icf`,
  `who_fic::ichi` paths resolve and a known-good code parses through each.
- `Classification` `FromStr`/`Display` round trip.
- `From<subcrate error> for FicError` conversions preserve variant meaning.
- `tests/serde.rs`: serde round trips through the umbrella re-export paths.
- Feature power set builds in CI (see architecture spec).
