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
| `serde` | no | forwards to each enabled subcrate's `serde` feature |

```toml
[features]
default = ["icd", "icf", "ichi"]
icd = ["dep:who-fic-icd"]
icf = ["dep:who-fic-icf"]
ichi = ["dep:who-fic-ichi"]
serde = ["who-fic-icd?/serde", "who-fic-icf?/serde", "who-fic-ichi?/serde"]
```

The crate must compile with `--no-default-features` (it is then just the
common module).

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

- `Display` prints the conventional short name: `"ICD-10"`, `"ICD-11"`,
  `"ICF"`, `"ICHI"`.
- `FromStr` accepts the short names case-insensitively, with and without the
  hyphen (`"icd11"`, `"ICD-11"`).
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

Implements `Display` + `std::error::Error`. Subcrates define structurally
identical error types of their own (to remain independent of the umbrella);
`who-fic` provides `From` conversions from each subcrate error to `FicError`
(gated on the matching feature) so multi-classification callers can unify on
one error type.

### `AnyCode` (deliberately deferred)

A `who_fic::AnyCode` enum wrapping any classification's code type is
plausible but not part of the initial scope; add it only when a concrete
consumer needs it. Recorded here so the decision is visible.

## Documentation

Crate-level rustdoc shows one parse-and-inspect example per classification,
each gated with the right `cfg(feature = ...)` so docs build under any
feature set.

## Tests

- Compile-and-use smoke test per feature: `who_fic::icd`, `who_fic::icf`,
  `who_fic::ichi` paths resolve and a known-good code parses through each.
- `Classification` `FromStr`/`Display` round trip.
- `From<subcrate error> for FicError` conversions preserve variant meaning.
- Feature power set builds in CI (see architecture spec).
