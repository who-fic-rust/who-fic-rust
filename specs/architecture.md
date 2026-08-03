# Spec: Architecture and conventions

Workspace-wide decisions that every crate in this repository follows.

## Workspace

The repository root is a virtual Cargo workspace (no root package):

```toml
[workspace]
resolver = "3"
members = ["who-fic", "who-fic-icd", "who-fic-icf", "who-fic-ichi"]
```

Shared metadata lives in `[workspace.package]` (version, edition, license,
repository, authors, rust-version) and each crate inherits it with
`field.workspace = true`. All four crates version together (lockstep
versioning) until there is a concrete reason not to.

- Edition: 2024
- License: MIT OR Apache-2.0 (code only; see "Licensing" below)
- MSRV: pinned in `rust-version` before first publish (Phase 6); until then,
  latest stable is assumed.

New subcrates are added as workspace members named with their parent crate's
name as prefix (e.g. `who-fic-icd-api`).

## Code-type conventions

Every classification code type in every crate:

1. Is an owned, immutable value type wrapping validated content; construction
   only via parsing/`TryFrom` — no public constructor that skips validation.
2. Implements `FromStr` (the canonical parser), `TryFrom<&str>`,
   `Display` (the canonical formatter), `Debug`, `Clone`, `PartialEq`, `Eq`,
   `Hash`, `PartialOrd`, `Ord`.
3. Provides `as_str(&self) -> &str` returning the canonical string form.
4. Round-trips: `T::from_str(s).unwrap().to_string()` equals the canonical
   form of `s`, and parsing the canonical form is identity.
5. Parsing is case-insensitive on input where the classification prints
   uppercase (ICD, ICHI) — canonical form is uppercase. ICF component letters
   are lowercase by convention and canonicalized to lowercase.
6. Whitespace is not trimmed by parsers; callers trim. (Keeps parsers exact
   and composable.)

**Validation depth:** parsers validate *syntax and structure* (allowed
characters, lengths, axis/qualifier structure, chapter ranges). They do not
validate *existence* (whether WHO has assigned the code) — that requires WHO
data and belongs to future data/API subcrates.

## Errors

- `who-fic` defines `FicError`, the shared parse-error enum
  (see [who-fic.md](who-fic.md)).
- Subcrates each define their own error type (e.g. `Icd10ParseError`) so they
  stay independent of the umbrella crate; variants follow the same shape
  (empty input / invalid length / invalid character with position / invalid
  structure with reason).
- All error types implement `std::error::Error + Display + Debug + Clone +
  PartialEq` and are `#[non_exhaustive]`.

## Features

Per subcrate:

- `serde` — derives `Serialize`/`Deserialize` for all public types. Codes
  serialize as their canonical string (via `Display`/`FromStr`), not as
  structs.
- `std` is not a feature: crates target `std` for now. `no_std` (with
  `alloc`) is a possible later addition; avoid gratuitous `std`-only
  dependencies in core code to keep the door open.

Umbrella crate `who-fic`:

- `icd`, `icf`, `ichi` — each pulls in and re-exports the matching subcrate.
  All three are default features.
- `serde` — forwards to every enabled subcrate's `serde` feature
  (`"who-fic-icd?/serde"` syntax).

## Dependencies

- Core crates: zero required dependencies. `serde` (with `derive`) optional.
- No `regex` dependency for code parsing — the grammars are small and fixed;
  hand-written matchers are faster to compile, dependency-free, and give
  better error positions.
- Dev-dependencies may include `proptest` for property tests.
- Heavier dependencies (HTTP, async, file formats) only in future subcrates.

## Testing

- Unit tests per module: accept-lists of known-good codes, reject-lists of
  known-bad codes (with the expected error variant).
- Property tests: parse→format→parse round trip; arbitrary strings never
  panic the parser.
- Doc tests: every public item has at least one rustdoc example that
  compiles and runs.
- Umbrella crate: build/test under the feature power set (`cargo hack
  --feature-powerset` in CI, or an explicit matrix).

## CI (Phase 6)

GitHub Actions on push/PR: `cargo fmt --check`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo test --workspace --all-features`,
feature matrix for `who-fic`, `cargo doc --no-deps` with
`RUSTDOCFLAGS="-D warnings"`, MSRV check once pinned.

## Licensing constraint (important)

WHO owns the copyright on classification *content* (code titles,
descriptions, inclusion/exclusion notes, the complete tabular lists).
This repository must not vendor or embed that content. What ships here:

- Code **syntax** rules (formats, alphabets, lengths, separators)
- **Structural** facts (components, axes, qualifier scales, chapter ranges
  and their leading characters)
- Small numbers of individual codes in tests and docs as factual examples

Anything requiring the full classification data (titles, existence checks,
search) is out of scope for these crates and is delegated to future
data-loader or API-client subcrates where the *user* supplies WHO-licensed
data or credentials.
