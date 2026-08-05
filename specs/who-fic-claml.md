# Spec: `who-fic-claml`

A parser for **ClaML** (Classification Markup Language), the XML format
standardized as ISO 13120 for exchanging healthcare classification
systems. WHO distributes ICD-10 in ClaML; other organizations (e.g.
national ICD-10 modifications) use it too, so this crate models the
general ClaML structure rather than anything ICD-10-specific — the ICD-10
adaptation (mapping `Class` entries to `Icd10Code`) lives in
`who-fic-icd`'s optional `claml` feature (see `specs/who-fic-icd.md`).

Per the workspace's licensing stance ([architecture.md](architecture.md)):
this crate parses XML the *user* supplies (a file they obtained themselves
under WHO's or the relevant maintainer's terms) — it does not fetch,
bundle, or embed classification content.

## Format (verified against the public ClaML DTD and reference examples,
2026-08)

Root element `<ClaML version="...">`, containing (among other
housekeeping elements this crate does not need to model in detail —
`Title`, `Identifier`, `ClassKinds`, `RubricKinds`) a flat list of `Class`
elements — the hierarchy is expressed by reference, not by XML nesting.
(The parser accepts any root element name and does not model the
`version` attribute; an empty or self-closing root yields an empty
document, not an error.)

```xml
<Class code="A00" kind="category">
  <SuperClass code="A00-A09"/>
  <SubClass code="A00.0"/>
  <SubClass code="A00.1"/>
  <Rubric kind="preferred">
    <Label xml:lang="en">Cholera</Label>
  </Rubric>
  <Rubric kind="inclusion">
    <Label xml:lang="en">Cholera due to Vibrio cholerae</Label>
  </Rubric>
</Class>
```

- `Class` has a `code` attribute (the classification code; required — a
  `Class` without one is a `MissingAttribute` error) and a `kind`
  attribute (e.g. `chapter`, `block`, `category` — open-ended per the
  file's own `ClassKinds` declaration; treat as a string; optional in
  this parser, defaulting to `""` — same for `Rubric`'s `kind`).
- `SuperClass code="..."` references the parent `Class` by code. A `Class`
  normally has zero (root-level chapters) or one `SuperClass`; the DTD
  permits more than one in principle (multiple-inheritance classifications)
  — the parser collects all `SuperClass` elements present, not just the
  first. **Known limitation (tracked in tasks.md's backlog):**
  `SuperClass`/`SubClass` are recognized only in self-closing form
  (`<SuperClass code="..."/>`); the equivalent — and equally well-formed —
  start/end-tag pair form (`<SuperClass code="..."></SuperClass>`) is
  silently skipped, losing that hierarchy link.
- `SubClass code="..."` elements are a redundant forward-reference list of
  children; useful for hierarchy traversal without a second pass, but
  derivable from `SuperClass` alone. Parse them, but don't treat them as
  more authoritative than `SuperClass`.
- `Rubric kind="...">` groups one or more `Label xml:lang="...">text
  </Label>` — `kind="preferred"` is the canonical title; other observed
  kinds include `inclusion`, `exclusion`, `definition`, `coding-hint`, etc.
  This crate should capture *any* `Rubric`/`Label` generically as
  `(kind, lang, text)` tuples rather than hard-coding the known kinds,
  since the set is classification-defined, not fixed by the format.
- ICD-10-specific `ModifierClass`/`Modifier` elements (used for shared
  4th/5th-character subclassification schemes) exist in the DTD; this
  crate should parse them into a generic form (a `ModifierClass` has a
  `code`, its own `Rubric`s, and contains `Modifier code="...">` entries
  with their own `Rubric`s) without attempting to model ICD-10's specific
  modifier semantics — that belongs in `who-fic-icd` if ever implemented.

## Types

- `ClamlDocument` — top-level parsed document: `title() -> Option<&str>`,
  `classes() -> &[Class]`, `modifier_classes() -> &[ModifierClass]`; also
  derives `Default` (an empty document). Parsed via `from_str` — the
  `std::str::FromStr` trait impl, so callers `use std::str::FromStr`
  (unlike `who-fic-linearization`, where `from_str` is inherent) — or
  `ClamlDocument::from_reader(impl std::io::Read)`, which reads the whole
  stream into a `String` first (the pull parser streams internally, but
  `from_reader` is not incremental).
- `Class` — `code() -> &str`, `kind() -> &str`, `super_classes() -> &[String]`
  (codes), `sub_classes() -> &[String]` (codes), `rubrics() -> &[Rubric]`,
  plus a convenience `preferred_label(lang: &str) -> Option<&str>` that
  finds the `kind="preferred"` `Rubric`'s `Label` for the given language.
- `Rubric` — `kind() -> &str`, `labels() -> &[Label]`.
- `Label` — `lang() -> &str` (from `xml:lang`, defaulting to `"en"` if
  absent — WHO's exports are English-only in practice), `text() -> &str`.
  ClaML labels can contain simple inline markup in real files (e.g. `<Para>`
  paragraphs, cross-reference elements); this crate extracts the
  concatenated text content (trimmed; unknown named entities are passed
  through verbatim as `&name;`) and does not preserve markup structure —
  a deliberate, documented simplification.
- `ModifierClass` / `Modifier` — generic form per the format section above.
- `ClamlError` (`#[non_exhaustive]`, `Debug + Clone`, `std::error::Error +
  Display`; no `PartialEq`, since `quick_xml::Error` has none) — variants
  `Xml { source, position }`, `MissingAttribute { element, attribute,
  position }`, `UnexpectedEof { element, position }`; positions are byte
  offsets from the underlying XML parser, and `Error::source` exposes the
  wrapped `quick_xml::Error`. Note this makes `quick-xml` a
  *semver-public* dependency: bumping its major version is a breaking
  change for this crate.

## Implementation note

Use a streaming/pull XML parser (e.g. `quick-xml`, a well-established
low-dependency crate) rather than hand-rolling XML parsing — unlike the
small fixed-grammar codes elsewhere in this workspace, general XML is not
a good fit for a hand-written parser. `quick-xml` becomes this crate's
one required dependency (the other core crates have zero required
dependencies — `serde` everywhere is optional; this is the deliberate,
documented exception because the format itself demands a real XML
parser).

## `serde`

Optional feature; derive `Serialize`/`Deserialize` on `ClamlDocument`,
`Class`, `Rubric`, `Label`, `ModifierClass`, `Modifier`.

## Tests

Use a small hand-written ClaML fixture (a few `Class` elements reflecting
the shape above, using ICD-10 codes already used as examples elsewhere in
this workspace, e.g. `A00`) — do not vendor WHO's actual ICD-10 ClaML
export. Cover: a class with a preferred rubric and an inclusion rubric,
a class with multiple `SubClass` entries, a class with more than one
`SuperClass`, a `ModifierClass`/`Modifier` pair, malformed XML (unclosed
tag), a `Class` missing its `code` attribute. All tests are inline unit
tests in `src/lib.rs` (no `tests/` directory; this crate has no property
tests).
