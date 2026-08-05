# Spec: `who-fic-linearization`

A parser for WHO's **Simplified Linearization Output** format: the
tab-separated export WHO's ICD-11 Maintenance Platform (`icd.who.int/dev11`)
produces for ICD-11 linearizations (MMS — Mortality and Morbidity
Statistics), ICF, and ICHI. All three classifications are exported from the
same platform in the same base tabular shape, which is why this is one
general-purpose crate rather than three copies of the same parser.

This crate is **format-only**: it turns rows of the export into a typed
`LinearizationRow` struct. It does not know about ICD/ICF/ICHI code syntax
or semantics — that mapping lives in each classification crate's own
optional `linearization` feature (see `specs/who-fic-icd.md`,
`specs/who-fic-icf.md`, `specs/who-fic-ichi.md`), which depends on this
crate and adapts `LinearizationRow`s into that classification's typed codes.

Per the workspace's licensing stance ([architecture.md](architecture.md)):
this crate parses a `Read`/`str` the *user* supplies (a file they downloaded
themselves from `icd.who.int/dev11/downloads`, under WHO's terms) — it does
not fetch, bundle, or embed WHO's classification content.

## File shape (verified against real downloads, 2026-08)

Downloaded as a `.zip` from `icd.who.int/dev11/downloads` (e.g.
`LinearizationMiniOutput-MMS-en.zip`, `...-ICF-en.zip`, `...-ICHI-en.zip`),
each containing a `.txt` (this crate's target) and an `.xlsx` (out of
scope — the crate does not parse Excel).

The `.txt` is:
- UTF-8 encoded, with a leading byte-order mark (BOM) on the first line —
  the parser must strip it.
- Tab-separated (`\t`), one row per line, first line is the header.
- Fields are inconsistently quoted CSV-style (double quotes, `""` for an
  embedded quote) — `Title` and `BrowserLink` are always quoted in
  practice, most other fields are bare. The parser must handle a field
  being either quoted-CSV-style or bare, not assume one or the other.
- Trailing empty fields on a line may be omitted (short line) rather than
  present as empty tab-separated slots — the parser must not require every
  row to have exactly the header's column count. Missing trailing columns
  default per field type: `Option`-typed fields to `None`, booleans to
  `false`, integers to `0` — so for the non-`Option` fields
  (`is_residual`, `is_leaf`, `depth_in_kind`,
  `no_of_non_residual_children`) a short line is indistinguishable from
  an explicit `False`/`0`.
- Quoted-vs-bare is detected from the field's first character only: a
  field starting with `"` is parsed as quoted-CSV (so a *bare* field that
  merely begins with a quote is a parse error, not taken literally).
  Boolean fields parse case-insensitively (`True`/`TRUE`/`true`). Blank
  lines between rows are skipped. Quoted fields cannot contain embedded
  newlines — reading is line-oriented.
- Columns are read *positionally*; the header is consulted only to detect
  the MMS layout (presence of a `Grouping1` column, done lazily on the
  first row read). Column *names* are not otherwise validated — a file
  with the right column count but reordered columns parses silently into
  the wrong fields.

### Header/columns

Common to all three classifications (MMS, ICF, ICHI):

| # | Column | Notes |
|---|---|---|
| 1 | `Foundation URI` | e.g. `http://id.who.int/icd/entity/257068234`; **empty** for residual (`.Y`/`.Z`) rows |
| 2 | `Linearization (release) URI` | always present; residual rows have a `/other` or `/unspecified` suffix |
| 3 | `Code` | empty for `chapter`/`block` rows; the classification's code for `category` rows |
| 4 | `BlockId` | empty for `category`/`chapter` rows; e.g. `BlockL1-1A0` for `block` rows |
| 5 | `Title` | quoted; leading repeated `"- "` markers encode depth (redundant with `DepthInKind`) — strip them when exposing the title |
| 6 | `ClassKind` | one of `chapter`, `block`, `category` (open-ended — treat as a string, not a closed enum, in case WHO adds kinds) |
| 7 | `DepthInKind` | integer, depth within its `ClassKind` |
| 8 | `IsResidual` | `True`/`False` |
| 9 | `PrimaryLocation` | `True`/`False`/empty |
| 10 | `ChapterNo` | e.g. `01`; empty for ICF/ICHI in practice |
| 11 | `BrowserLink` | an Excel `=hyperlink(...)` formula string — exposed after the standard CSV unquoting (outer quotes stripped, `""` → `"`) but not otherwise parsed |
| 12 | `isLeaf` | `True`/`False` |
| 13 | `noOfNonResidualChildren` | integer |
| trailing | `Version:<timestamp>` | the header's last column name embeds the export timestamp; there is no corresponding data column — this is a header-only artifact, not a real 14th field |

**MMS only** — 5 additional columns after column 13:

| # | Column |
|---|---|
| 14 | `Primary tabulation` (`True`/`False`/empty) |
| 15–19 | `Grouping1`…`Grouping5` (block-id references for postcoordination groupings; empty when unused) |

## Types

- `LinearizationRow` (`Clone + Debug + PartialEq + Eq`) — one row, all
  fields above as typed accessors
  (`foundation_uri() -> Option<&str>`, `linearization_uri() -> &str`,
  `code() -> Option<&str>`, `block_id() -> Option<&str>`,
  `title() -> &str` — depth markers stripped, `class_kind() -> &str`,
  `depth_in_kind() -> u32`, `is_residual() -> bool`,
  `primary_location() -> Option<bool>`, `chapter_no() -> Option<&str>`,
  `browser_link() -> Option<&str>`, `is_leaf() -> bool`,
  `no_of_non_residual_children() -> u32`,
  `primary_tabulation() -> Option<bool>` and
  `groupings() -> &[String]` — the last two `None`/empty unless the file
  had the MMS-only columns).
- `LinearizationReader<R>` — streaming row iterator
  (`Iterator<Item = Result<LinearizationRow, LinearizationError>>` where
  `R: std::io::Read`; the bound sits on the impls, the struct itself is
  unbounded) so callers don't have to load a multi-megabyte export into
  memory at once. The header line is consumed lazily by the first
  `next()`. Constructors: `::from_reader(R)`, plus an *inherent* (not
  trait) `LinearizationReader::from_str(&str)` defined only on the
  concrete `LinearizationReader<Cursor<Vec<u8>>>` — it copies the input,
  trading an allocation for a simple return type.
- `LinearizationError` (`#[non_exhaustive]`, `Clone + Debug + PartialEq +
  Eq`, `std::error::Error + Display`) — variants `Io { line, message }`,
  `UnterminatedQuotedField { line }`, `TrailingDataAfterQuotedField {
  line }`, `InvalidBoolean { line, field, found }`, `InvalidInteger {
  line, field, found }`, with `line() -> usize` reporting the 1-based
  line number (the header counts as line 1). This crate is not a WHO-FIC
  *code* crate, so the shape deliberately doesn't match `FicError`
  variant-for-variant.

## Non-goals

- No code-syntax validation (that's each classification crate's job).
- No `.xlsx` parsing.
- No network fetching of the export.
- No semantic understanding of `Grouping1`–`5` beyond exposing them as
  strings — full postcoordination semantics for ICD-11 remain out of scope
  workspace-wide (see plan.md risks).

## `serde`

Optional feature; derive `Serialize`/`Deserialize` on `LinearizationRow`
(it's a plain data struct here, unlike the code types elsewhere in the
workspace — there's no canonical single-string form to round-trip
through). Side effect worth knowing: `Deserialize` is the one way to
construct a `LinearizationRow` outside this crate — there is otherwise no
public constructor. Round-trip tests live in `tests/serde.rs`.

## Tests

Use small, hand-written fixtures (a handful of representative lines in the
real header + row shape documented above) — do not vendor WHO's actual
export files. Cover: a `chapter` row, a `block` row, a `category` row, a
residual (`.Y`/`.Z`) row with empty `Foundation URI`, a short line missing
trailing columns, an MMS-style row with `Grouping` columns present, an
ICF/ICHI-style row without them, a malformed line (wrong quoting) — with
the expected `LinearizationError` and line number. `tests/proptest.rs`
adds property tests (parser never panics on arbitrary input, among
others).
