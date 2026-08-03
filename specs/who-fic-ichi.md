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

The section is determined by the target's leading character range per the
published ICHI tabulation. Encode the range table as a structural fact with
boundary unit tests; expose `IchiCode::section()` and `Target::section()`.
(If the beta's ranges prove unstable, return `Option<Section>` and document.)

### Extension codes

ICHI defines extension codes (e.g. laterality, quantifiers) that refine
intervention codes, conceptually similar to ICD-11 extensions. Initial
scope: **documented but not implemented** — reserve a `extension` module
with the design sketch in rustdoc, implement when the post-beta shape is
firm. Recorded in tasks.md backlog.

## Errors

`IchiParseError` per the shared error shape, plus an `axis:
Option<Axis>` field (`Axis = Target | Action | Means`) identifying where in
the dotted form parsing failed. `who-fic` converts to `FicError`.

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
