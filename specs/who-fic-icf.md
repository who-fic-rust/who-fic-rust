# Spec: `who-fic-icf`

International Classification of Functioning, Disability and Health (ICF):
the WHO framework for describing health and health-related states in terms
of functioning and disability, complementing ICD's disease view.

Reference: https://www.who.int/standards/classifications/international-classification-of-functioning-disability-and-health

All types follow the workspace code-type conventions
([architecture.md](architecture.md)). ICF's canonical form uses a
**lowercase** component letter (`b280`, not `B280`).

## Structure of the classification

ICF codes have three parts: a component letter, a numeric hierarchy, and
optional qualifiers.

### `Component`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Component {
    BodyFunctions,          // 'b'
    BodyStructures,         // 's'
    ActivitiesAndParticipation, // 'd'
    EnvironmentalFactors,   // 'e'
}
```

With `letter(&self) -> char` and parsing from `b`/`s`/`d`/`e`
(case-insensitive input, lowercase canonical). Not `#[non_exhaustive]`:
the four components are fixed by the ICF's design. (Personal factors are
part of the ICF model but are *not classified* — no codes — so no variant.)

### Hierarchy: `IcfCode`

```
code    = component digits
component = "b" / "s" / "d" / "e"
digits  = 1 digit            ; chapter        (level 1), e.g. b2
        / 3 digits           ; second level,           e.g. b280
        / 4 digits           ; third level,            e.g. b2801
        / 5 digits           ; fourth level,           e.g. b28010
```

Two digits is invalid (there is no 2-digit level). API:

- `component(&self) -> Component`
- `level(&self) -> Level` where `Level` is
  `Chapter | SecondLevel | ThirdLevel | FourthLevel`
- `parent(&self) -> Option<IcfCode>` — truncate one level
  (`b28010` → `b2801` → `b280` → `b2`; chapter has no parent)
- `chapter(&self) -> IcfCode` — truncate to level 1
- `is_ancestor_of` / `is_descendant_of`

### Qualifiers

Qualifiers attach after the numeric part and carry the actual assessment.
The **generic scale** for a single qualifier digit:

| Digit | Meaning |
|---|---|
| 0 | NO problem (none, absent, negligible) |
| 1 | MILD problem |
| 2 | MODERATE problem |
| 3 | SEVERE problem |
| 4 | COMPLETE problem |
| 8 | not specified |
| 9 | not applicable |

(5–7 are invalid.) Model as `Qualifier` enum or validated newtype.

Qualifier structure differs per component — this is the crux of ICF and the
type system should enforce it:

- **Body functions (`b`)** — one qualifier: extent of impairment.
  `b280.2`
- **Body structures (`s`)** — up to three positional qualifiers: extent of
  impairment, nature of change, location. `s730.312`
- **Activities and participation (`d`)** — first qualifier = performance,
  second = capacity (without assistance); up to four positions are defined
  in the ICF but the first two are the ones in standard use; support 1–4,
  document positions 3–4 as optional-use. `d450.12`
- **Environmental factors (`e`)** — one qualifier that is either a
  **barrier** (separator `.`) or a **facilitator** (separator `+`):
  `e150.2` (moderate barrier) vs `e150+2` (moderate facilitator).

Design: `QualifiedIcfCode` = `IcfCode` + component-appropriate qualifier
payload, with the parser rejecting mismatches (e.g. `+` on a `b` code, three
qualifiers on a `b` code). A bare `IcfCode` (no qualifiers) remains valid on
its own — WHO recommends codes be used with qualifiers, but the unqualified
form is what appears in the tabulation itself.

## Errors

`IcfParseError` per the shared error shape; `who-fic` converts to
`FicError`.

## `serde`

Optional feature; canonical-string serialization for `IcfCode` and
`QualifiedIcfCode`.

## Tests

- Accept-list: `b2`, `b280`, `b2801`, `b28010`, `s730.312`, `d450.12`,
  `e150+2`, `e150.2`.
- Reject-list: `b28` (2 digits), `x280` (bad component), `b280.5` (invalid
  qualifier digit), `b280+2` (facilitator on non-`e`), `s730.3124` +
  overlong qualifier strings, empty input.
- Hierarchy: `parent`/`chapter`/ancestor tests across all levels.
- Property tests: round trip; arbitrary input never panics; `parent` is
  monotone in level.
