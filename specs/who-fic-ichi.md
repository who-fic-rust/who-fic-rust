# Spec: `who-fic-ichi`

International Classification of Health Interventions (ICHI): the WHO
classification for reporting and analyzing health interventions — curative,
preventive, diagnostic, rehabilitative, and public-health actions — across
all sectors of the health system.

Reference: https://www.who.int/standards/classifications/international-classification-of-health-interventions

**Stability note:** ICHI's latest public release is a beta (Beta-3). The
axis architecture below is stable, but code details may change before final
adoption. Crate docs must state this; enums that mirror WHO value sets are
`#[non_exhaustive]`.

All types follow the workspace code-type conventions
([architecture.md](architecture.md)); canonical form is uppercase.

## Structure of the classification

Every ICHI intervention code is built from three axes:

- **Target** — the entity on which the action is carried out
  (anatomy, body function, activity domain, environment, …)
- **Action** — the deed done by the actor to the target
  (e.g. excision, education, assessment)
- **Means** — the processes and methods by which the action is carried out
  (e.g. open approach, endoscopic, instrument)

### Code syntax

```
code   = target "." action "." means
target = 3 alnum          ; e.g. "KAB"
action = 2 alnum          ; e.g. "DB"
means  = 2 alnum          ; e.g. "AD"
alnum  = A–Z / 0–9
```

Example: `KAB.DB.AD`. The separators are literal dots; total canonical
length is 9.

### Types

- `Target`, `Action`, `Means` — validated newtypes over their fixed-length
  uppercase-alphanumeric syntax, each with `as_str()`. Individual axis codes
  are meaningful on their own (the axes are published as standalone value
  sets), so they are public types, not internal fields.
- `IchiCode` — the composed `TTT.AA.MM` intervention code:
  - `target(&self) -> &Target`, `action(&self) -> &Action`,
    `means(&self) -> &Means`
  - `from_parts(Target, Action, Means) -> IchiCode` (infallible once the
    parts exist — composition has no extra constraints at the syntax level)
  - `FromStr` parses the dotted form and reports which axis failed
    (error carries an `Axis` discriminant).

### `Section`

ICHI groups interventions into three sections by what the target is:

```rust
#[non_exhaustive]
pub enum Section {
    BodySystemsAndFunctions,
    ActivitiesAndParticipationDomains,
    Environment,
}
```

The section is nominally determined by the target's leading character
range per the published ICHI tabulation. In practice no verified,
publicly-sourceable leading-character-to-chapter table could be confirmed
for Beta-3 (anatomy targets, ICF-derived body-function targets, and other
categories interleave on shared leading letters), so
`Target::section(&self) -> Option<Section>` and `IchiCode::section()`
currently always return `None`, per the spec's explicit fallback clause.
Rustdoc on `section()` explains why and what would be needed to upgrade it
(a confirmed WHO Beta-3 leading-character/chapter table).

### Extension codes

ICHI defines extension codes (e.g. laterality, quantifiers) that refine
intervention codes, conceptually similar to ICD-11 extensions. Initial
scope: **documented but not implemented** — reserve a `extension` module
with the design sketch in rustdoc, implement when the post-beta shape is
firm. Recorded in tasks.md backlog.

## Errors

`IchiParseError` per the shared error shape, plus an `axis: Option<Axis>`
field on every variant (including `Empty`) (`Axis = Target | Action |
Means`) identifying which dot-segment of the code failed to parse; `None`
when the failure isn't attributable to a single segment (e.g. the whole
input was empty, or the dotted structure itself was wrong — missing
separators, wrong segment count). `who-fic`'s `From<IchiParseError> for
FicError` conversion drops the `axis` field (there is no equivalent slot in
`FicError`); code that needs axis-level detail should match on
`IchiParseError` directly instead of going through `FicError`.

## Data loading (optional)

**`linearization` feature** (module `linearization`, depends on
[`who-fic-linearization`](who-fic-linearization.md)): adapts
`LinearizationRow`s from WHO's ICHI "Simplified Linearization Output"
export into a lookup from `IchiCode` to title. Follows the shared
"Data-loading index conventions" in [architecture.md](architecture.md)
(`BTreeMap`-backed, `IchiClassEntry` carries its own `.code()`,
`.iter()`/`IntoIterator` yield entries in ascending code order) — same
shape as `who-fic-icf`'s `linearization` feature.
`IchiLinearizationIndex::from_rows(impl Iterator<Item = Result<LinearizationRow,
LinearizationError>>) -> Result<Self, IchiLinearizationError>` — takes
exactly what a `LinearizationReader` yields; a reader-level `Err`
propagates immediately. Rows whose `Code` doesn't
parse as an `IchiCode` are skipped rather than treated as fatal (in
practice this includes a small number of `(proposed)` Beta-3 entries still
using a placeholder `??` target). Retaining the block-title hierarchy
above each code — which a caller could use to derive real section
groupings, a path to eventually replacing `Section`'s current always-`None`
fallback (see above) with real data — is a documented, not-yet-implemented
future extension of this index, not part of its initial scope.

## `serde`

Optional feature; `IchiCode` and the axis types serialize as canonical
strings.

## Tests

- Accept-list: `KAB.DB.AD` and a small sample of real Beta-3 codes across
  all three sections.
- Reject-list: wrong segment lengths (`KA.DB.AD`, `KAB.D.AD`), missing or
  wrong separators (`KAB-DB-AD`, `KABDBAD`), lowercase canonicalization
  check, invalid characters, empty input, trailing garbage.
- `from_parts` ∘ accessors is identity; dotted round trip.
- Section boundary tests for the target ranges.
- Property tests: round trip; arbitrary input never panics.
