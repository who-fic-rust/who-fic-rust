# Lessons from this project's development

Specific mistakes made and caught while building this workspace, kept
here so they don't get repeated. Each one is a real incident, not a
hypothetical.

## Parallel agents drift on anything not explicitly specified

Most of this workspace's crates were built by separate agent runs working
concurrently, each given a detailed, self-contained prompt and no
visibility into sibling agents' code. This works well for genuinely
independent work, but **any design choice the spec didn't pin down
exactly will be made differently by different agents**, even when the
crates are conceptually parallel (e.g. "adapt a linearization export into
a lookup" for ICD-11, ICF, and ICHI). Three real instances:

1. **`from_rows` signature**: `who-fic-ichi`'s adapter accepted
   `impl Iterator<Item = Result<LinearizationRow, LinearizationError>>`
   (matching what `LinearizationReader` actually yields, letting reader
   errors propagate directly); `who-fic-icf`'s took a plain
   `impl Iterator<Item = LinearizationRow>` instead (forcing callers to
   pre-unwrap). Neither prompt had specified the exact signature — both
   were reasonable, they just didn't match. Caught and harmonized before
   publishing (to `who-fic-icf`'s shape, matching the majority).

2. **`HashMap` vs `BTreeMap`, `iter()` shape, `IntoIterator` presence**:
   all four data-loading `*Index` types (`who-fic-icd`'s two,
   `who-fic-icf`'s, `who-fic-ichi`'s) needed to do the same job — a
   code-keyed lookup with iteration — and ended up with three different
   internal map types and two different `iter()` return shapes (bare
   `&ClassEntry` vs. `(&Code, &ClassEntry)` tuples, depending on whether
   that crate's `ClassEntry` happened to carry its own `code()`), with
   `IntoIterator` present on only one of the four. This one shipped in a
   published release (0.2.0) before being caught in a later harmonization
   pass — a real inconsistency users could have hit.

3. **Section-derivation attempt**: less a drift than a good example of an
   agent correctly declining to guess — asked to encode an ICHI target
   leading-character-to-section table, the agent researched what it could
   verify, found no confirmable source, and returned `Option<Section>` /
   always `None` with an honest explanation rather than fabricating a
   plausible-looking table. This is the right failure mode; the two
   examples above are the wrong one.

**Takeaway**: when a spec describes "the same kind of thing" happening in
multiple crates (multiple code types, multiple adapters, multiple error
types), either (a) pin down the exact shape in the spec/prompt so there's
nothing left to drift, or (b) explicitly schedule a harmonization pass
after all the parallel work lands, before publishing — don't assume
"each one individually passed its own tests" implies "they're consistent
with each other." `specs/architecture.md`'s "Data-loading index
conventions" section exists specifically so instance 2 can't recur.

## Local verification can pass while CI fails — verify the actual CI command

This repository's CI failed on **every single push** from its first
commit through several publish cycles, and this went unnoticed for a long
time because local spot-checks always used a narrower command than what
CI actually runs:

- Local check used: `cargo build -p who-fic --no-default-features`
  (library only).
- CI runs: `cargo test --workspace --no-default-features` (also compiles
  every example and test target across the whole workspace).

The gap: `who-fic/examples/readme.rs` referenced `who_fic::icd`/`icf`/
`ichi` unconditionally. Those are optional, default-*on* features, so
`cargo build` (lib only) never noticed they could be off; `cargo test
--workspace --no-default-features` did, immediately, the moment anyone
actually ran it — which nobody had, against CI, until an explicit audit
pass checked `gh run list` and found three consecutive failing runs.

Separately, `cargo test --workspace --all-features` (which *had* been run
locally many times) still carried a latent issue that only showed up as a
warning, not a failure: every crate's `examples/readme.rs` produced an
identically-named `readme` binary, and Cargo warned about the output
filename collision ("this may become a hard error in the future") on
every single run — also unnoticed because the check was "no test
failures," not "read every warning in the output."

**Takeaway**: "I ran a similar command and it passed" is not the same
claim as "I ran the exact command CI runs." When in doubt, read
`.github/workflows/ci.yml` and copy the literal command. After pushing,
check `gh run list` — don't assume green.

## `#![warn(missing_docs)]` needs to be set explicitly, per crate

Four of the seven crates had it from the start; three (`who-fic-icd`,
`who-fic-ichi`, `who-fic-linearization`) didn't, simply because whichever
agent scaffolded each crate's `lib.rs` didn't happen to include it. Since
it's off by default in rustc, its absence is silent — nothing fails,
nothing warns, an undocumented public item just... doesn't get caught.
Caught by an explicit audit ("which crates have this lint set?"), not by
any test or CI failure, since there was nothing yet undocumented to
trigger it. Now set in all seven; if you scaffold an eighth crate, set it
there too, on purpose, don't assume it's inherited from somewhere.

## When researching an external format/API, verify against the primary source directly

The two format-parser crates (`who-fic-linearization`, `who-fic-claml`)
and the API client (`who-fic-icd-api`) all needed exact structural
details of things this project doesn't control (WHO's TSV export
columns, ClaML's XML element structure, WHO's ICD-API OpenAPI spec).
Rather than write specs from general knowledge or secondhand blog posts,
each was verified against a primary source before implementation started:
actual downloaded sample files (inspected, then deleted — not vendored)
for the TSV format, and WHO's own machine-readable OpenAPI document
(`https://id.who.int/swagger/v2/swagger.json`) fetched directly for the
API client. This caught real details that would have been easy to
hand-wave (e.g. the TSV's BOM, its Excel-style quoting inside a
tab-delimited file, the exact `codeinfo` response shape). Prefer this over
confident-sounding paraphrase, especially for anything a test fixture or
type definition will encode as fact.
