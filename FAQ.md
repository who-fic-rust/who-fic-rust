# FAQ

Specific questions that don't fit a walkthrough. Start with
[TUTORIAL.md](TUTORIAL.md) if you're new here; this is for "does it do
X" questions once you already know the shape of the workspace.

## Which crate do I need?

- Only one classification (ICD, ICF, or ICHI)? Depend on that crate
  directly (`who-fic-icd`, `who-fic-icf`, `who-fic-ichi`).
- More than one, or you want the shared `Classification`/`FicError`
  types? Depend on [`who-fic`](https://crates.io/crates/who-fic) instead
  — it re-exports the others behind features.
- Need a code's actual title, not just to validate its syntax? You also
  need that classification's data-loading feature (`claml` on
  `who-fic-icd` for ICD-10, `linearization` everywhere else) plus a WHO
  export file you download yourself, or `who-fic-icd-api` for a live
  lookup. See the next few questions.

## Can I look up what a code actually *means* — its title?

Not from the code-parsing crates alone, and deliberately so — see "Why
doesn't this bundle WHO's classification content?" below. Two ways to get
titles:

1. **Offline, from a file you download**: enable a crate's `claml` or
   `linearization` feature, get the matching export from WHO yourself
   (`https://icd.who.int/dev11/downloads` for the TSV linearization
   format; ICD-10 ClaML exports are distributed by WHO and national
   maintainers), and build an `*Index` (`Icd10ClamlIndex`,
   `Icd11LinearizationIndex`, `IcfLinearizationIndex`,
   `IchiLinearizationIndex`). See TUTORIAL.md section 5.
2. **Live, from WHO's own server**: `who-fic-icd-api`, with your own free
   WHO ICD-API credentials. See TUTORIAL.md section 6. ICD-11/MMS only —
   there's no live API equivalent in this workspace for ICF/ICHI/ICD-10.

## Why doesn't this bundle WHO's classification content?

WHO owns the copyright on it — titles, definitions, inclusion/exclusion
notes, the complete tabular lists. Redistributing that requires WHO's
permission, which this project doesn't have and isn't seeking. What ships
here is code *syntax and structure* (how a code is spelled, how the
hierarchy fits together, which qualifiers/axes exist) plus parsers for
formats *you* feed with *your own* WHO-obtained files or credentials.
Full rationale in `specs/architecture.md`'s "Licensing constraint"
section.

## Does a successfully-parsed code mean WHO has actually assigned it?

No. Every code type here validates *syntax* (right shape, right
characters, right structure) — not *existence*. `Icd11Code::from_str("1A99.9")`
succeeds if `"1A99.9"` is a syntactically well-formed ICD-11 code, whether
or not WHO has actually defined that specific code. Existence checking
needs WHO's data (a data-loading `*Index`, or `who-fic-icd-api`'s live
lookup) — see the two questions above.

## Is this validated for clinical, billing, or regulatory use?

**No — treat it as a syntax/structure library, not a certified coding
tool.** These crates implement WHO's published code grammars and
hierarchy rules as understood and tested by this project; they are not
WHO software, are not clinically validated, and existence/currency
checks require data or credentials *you* supply and are responsible for.
If you're building something where a wrong or stale code has real
consequences (billing, clinical decision support, regulatory reporting),
verify against WHO's own current authoritative sources, not just this
library's syntax checks.

## Why is ICD-10 handled so differently from ICD-11/ICF/ICHI?

Because WHO actually distributes them differently. ICD-11, ICF, and ICHI
all export from the same WHO Maintenance Platform in one shared
tab-separated "Simplified Linearization Output" format (`who-fic-linearization`
parses it). ICD-10 is instead distributed as ClaML XML
(`who-fic-claml` parses it) — this workspace doesn't know or claim why
WHO's own tooling draws that line, only that it verifiably does (checked
by downloading and inspecting real exports of each before designing the
parsers, not assumed).
See `specs/who-fic-linearization.md` and `specs/who-fic-claml.md`.

## What's the difference between `who-fic-linearization` and, say, `IcfLinearizationIndex`?

`who-fic-linearization` is format-only — it turns TSV rows into a generic
`LinearizationRow` struct with no idea what an ICD/ICF/ICHI code is.
`IcfLinearizationIndex` (in `who-fic-icf`, behind its `linearization`
feature) is the adapter: it depends on `who-fic-linearization` and maps
`LinearizationRow`s into `IcfCode`-keyed lookups, skipping rows that
aren't ICF-code-shaped. The same pattern repeats for
`Icd11LinearizationIndex` and `IchiLinearizationIndex`, and for ClaML with
`Icd10ClamlIndex`. See `specs/architecture.md`'s "Data-loading index
conventions".

## Why does `Icd10Code::chapter()` (and `Icd11Code::chapter()`) return `Option`?

Because WHO reserves some numeric sub-ranges within an otherwise-assigned
letter/chapter (e.g. ICD-10's `D49`, `E91`–`E99`) without assigning them
to any chapter. `None` means "this category isn't in any chapter's
assigned range" — a real, documented fact about the classification, not
a parse failure (the code itself still parses fine).

## Is this crate async? Does it need `tokio`?

Only `who-fic-icd-api` — the one crate that makes network calls. Every
other crate is synchronous, pure computation, with no dependency on an
async runtime at all (`who-fic-claml` needs `quick-xml` for XML parsing,
which is also sync). If you don't depend on `who-fic-icd-api`, nothing in
this workspace pulls in `tokio`.

## ICHI codes I expect aren't parsing / aren't in the export — is that a bug?

Maybe not — ICHI's latest public release is a **beta** (Beta-3). A small
number of entries in WHO's own export are still marked `(proposed)` and
use a placeholder `??` target axis instead of a real 3-character one;
`IchiCode::from_str` correctly rejects those (they're not valid ICHI
syntax yet), and `IchiLinearizationIndex` skips them leniently rather than
failing the whole build. If a code you expect to be valid genuinely isn't
parsing and isn't one of these placeholder entries, that may be a real
bug — please open an issue with the specific code.

## Does this support `no_std`?

Not currently. It's a documented possible future direction (see
`specs/architecture.md`'s "Features" section) — core crates avoid
gratuitous `std`-only dependencies to keep the door open, but nobody has
done the work to actually gate on it yet.

## What Rust version do I need?

1.85 (edition 2024's minimum), pinned in the workspace's `rust-version`
and checked by CI's `msrv` job on every push.

## Will a future release break my code?

All seven crates are pre-1.0 (`0.x.y`). Per Rust's semver convention for
pre-1.0 crates, a breaking change bumps the *minor* version (`0.2.0` →
`0.3.0`), not the patch version — so pin with `"0.3"` (meaning `^0.3.0`,
i.e. accept patch-level updates only) if you want protection from
breaking changes, not just `"0"`. This has already happened once (see
`CHANGELOG.md`'s `0.3.0` entry) when a cross-crate API inconsistency was
found and fixed before it could calcify.

## Where do I report a bug or ask something not covered here?

Open an issue using the templates in `.github/ISSUE_TEMPLATE/`. See
[CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution workflow.
