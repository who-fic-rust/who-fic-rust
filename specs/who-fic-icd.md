# Spec: `who-fic-icd`

International Classification of Diseases (ICD): the global standard for
recording and reporting diseases, disorders, injuries, and related health
conditions, for mortality/morbidity statistics and clinical documentation.

Reference: https://www.who.int/standards/classifications/classification-of-diseases

Two revisions are in wide use and both are in scope, as modules of this one
crate: **ICD-10** (legacy, still dominant in many systems) and **ICD-11**
(current, in effect since 2022). They have different code syntaxes and must
not share a code type.

```
who-fic-icd
├── icd10   module: Icd10Code, Icd10Chapter, Icd10ParseError
└── icd11   module: Icd11Code, Icd11Chapter, Cluster, Icd11ParseError
```

If a revision's scope grows substantially (data loaders, per-revision
tooling), promote its module to a subcrate (`who-fic-icd-10`,
`who-fic-icd-11`) re-exported here — not before.

All types follow the workspace code-type conventions
([architecture.md](architecture.md)); canonical form is uppercase.

## Module `icd10`

### Code syntax

```
code        = category [ "." subdivision ]
category    = letter digit digit            ; "A00" … "Z99"
subdivision = 1*2( digit / letter )         ; commonly one digit, e.g. "I63.9"
```

- `Icd10Code` accessors: `category(&self) -> &str` (3 chars),
  `subdivision(&self) -> Option<&str>`, `chapter(&self) -> Option<Icd10Chapter>`.
  `chapter()` returns `Option` rather than a bare `Icd10Chapter`: several
  numeric sub-ranges within an otherwise-assigned letter (e.g. `D49`,
  `E91`–`E99`, `K94`–`K99`) are reserved/unassigned by WHO, so a total
  function would have to fabricate a chapter for codes WHO has not
  assigned one to. `None` means "this category isn't in any chapter's
  assigned range," not "parsing failed."
- Dagger/asterisk markers (`†`, `*`) and national extensions (e.g. ICD-10-CM
  7-character codes) are **out of scope**; parser rejects them. Document this
  prominently — it is the most likely user surprise.

### `Icd10Chapter`

Enum of the 22 chapters (I–XXII). Mapping from code to chapter follows the
official category ranges (chapter I is `A00–B99`, chapter II `C00–D48`,
etc. — note chapters are *not* aligned to single letters; e.g. `D50` is
chapter III, `H00` vs `H60` split chapters VII/VIII). The full range table is
a structural fact and is encoded in the crate with a unit test per boundary.

## Module `icd11`

### Background

ICD-11 distinguishes the *Foundation* (a semantic network of entities with
URIs) from *linearizations*, chiefly the MMS (Mortality and Morbidity
Statistics), which is where codes live. This crate models MMS codes.
Foundation URIs/entity IDs are out of scope for the core crate (they belong
to the future `who-fic-icd-api` subcrate).

### Code syntax

```
stem       = chapterchar letter alnum alnum [ "." 1*2alnum ]
chapterchar= digit / letter                  ; identifies the chapter
alnum      = digit / letter
letter     = A–Z excluding "I" and "O"       ; excluded to avoid 1/0 confusion
```

- Stem codes are 4 characters before any subdivision: e.g. `8B20` (stroke),
  `CA40` (pneumonia), with subdivisions like `CA40.0`.
- The letters `I` and `O` never appear in ICD-11 codes; the parser rejects
  them with `InvalidCharacter`.
- The second character is always a letter (distinguishes ICD-11 codes from
  ICD-10 at a glance); the parser enforces this.
- Codes beginning with `X` are **extension codes** (severity, anatomy,
  temporality, etc.), used only in postcoordination. Represent as either a
  distinct `ExtensionCode` type or an `is_extension()` flag — decide at
  implementation time; spec preference: distinct type, so stem-only APIs are
  type-enforced.
- Terminal `.Y` means "other specified", terminal `.Z` means "unspecified":
  expose `is_residual_other(&self)` / `is_residual_unspecified(&self)`.

### `Icd11Chapter`

Enum of chapters 01–26 plus `V` (supplementary functioning assessment) and
`X` (extension codes). Derived from the code's leading character per the
official MMS chapter table (the leading character alone identifies the
chapter — WHO's reference guide states every code in a chapter shares the
same first character). Several leading characters (`0`, `T`, `U`, `W`, `Y`,
`Z`) are unassigned, so `Icd11Code::chapter(&self) -> Option<Icd11Chapter>`
returns `Option` for the same reason as `Icd10Code::chapter()` above.
Encoded with boundary unit tests.

### `ExtensionCode`

Codes starting with `X`, used only in postcoordination. A distinct type
from `Icd11Code`, sharing the same 4-char-plus-optional-subdivision
grammar (e.g. `XA00.6`). `Icd11Code::from_str` rejects `X`-prefixed input;
`ExtensionCode::from_str` requires it. Note: this crate's extension-code
grammar always requires a `.` before the subdivision; WHO's real extension
codes sometimes use a flat 6-character form without a dot (e.g. `XA0060`) —
that form is out of scope for now.

### Postcoordination clusters

ICD-11 codes combine into clusters:

- `&` joins a stem code with extension code(s): `8B20&XK9J`
- `/` joins multiple stem codes into one clinical concept:
  `NA07.1/8B20` (varies by convention; both separators occur in cluster
  strings)

`Cluster` and `ClusterStem` parse a cluster string into its parts and
format it back canonically: a `Cluster` is one or more `/`-separated
`ClusterStem`s, and each `ClusterStem` is one stem code plus zero or more
`&`-attached extension codes. **Syntax only**: whether a given extension is
*permitted* on a given stem requires WHO data and is out of scope (see
plan.md risks).

## Errors

`Icd10ParseError` and `Icd11ParseError`, per the shared error shape
(empty / invalid length / invalid character with position / invalid
structure). `who-fic` provides `From` conversions to `FicError`.

## Data loading (optional)

Two independent optional features add title/hierarchy lookups sourced from
WHO's own exports — the user supplies the file; this crate never bundles
WHO content (see [architecture.md](architecture.md)).

- **`claml` feature** (module `icd10::claml`, depends on
  [`who-fic-claml`](who-fic-claml.md)): adapts a parsed ClaML `Class` list
  into a lookup from `Icd10Code` to its preferred title and chapter
  membership. `Icd10ClamlIndex::from_document(&ClamlDocument) -> Result<Self,
  Icd10ClamlError>` builds the index (parsing each `Class`'s `code` as an
  `Icd10Code`, skipping/reporting non-ICD-10-shaped classes such as chapter
  or block entries that don't parse); `.title(&Icd10Code) -> Option<&str>`,
  `.get(&Icd10Code) -> Option<&Icd10ClassEntry>`.
- **`linearization` feature** (module `icd11::linearization`, depends on
  [`who-fic-linearization`](who-fic-linearization.md)): adapts
  `LinearizationRow`s from the MMS export into a lookup from `Icd11Code` to
  title, chapter, residual status, and — since MMS rows carry
  `Grouping1`–`5` — its block-grouping path. `Icd11LinearizationIndex::
  from_rows(impl Iterator<Item = Result<LinearizationRow, LinearizationError>>) -> Result<Self,
  Icd11LinearizationError>`; same `.title()`/`.get()` shape as the ICD-10
  index.

Both indexes are read-only, in-memory lookups built once from a
user-supplied export; they are not live queries against WHO's API (that's
the separate, unscheduled `who-fic-icd-api` subcrate).

## `serde`

Optional feature; all code types serialize as canonical strings.

## Tests

- Accept-lists: real published codes from both revisions (small factual
  sample: `A00`, `I63.9`, `U07.1`; `8B20`, `CA40.0`, `1A00`, `XK9J`, …).
- Reject-lists: ICD-10 codes fed to the ICD-11 parser and vice versa;
  codes containing `I`/`O` (ICD-11); wrong separators; empty/overlong input.
- Chapter-boundary tests for both chapter enums.
- Property tests: round trip; arbitrary input never panics.
