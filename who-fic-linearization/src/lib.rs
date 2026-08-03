//! Parser for WHO's "Simplified Linearization Output" tab-separated export
//! format, used by ICD-11 (MMS), ICF, and ICHI exports from
//! <https://icd.who.int/dev11/downloads>.
//!
//! This crate is format-only: it has no knowledge of ICD/ICF/ICHI code
//! syntax. Each classification crate in the WHO-FIC-Rust workspace has its
//! own optional feature that adapts [`LinearizationRow`] into that
//! classification's typed codes — see `specs/who-fic-linearization.md` in
//! the repository for the full format specification and
//! `specs/who-fic-icd.md` / `who-fic-icf.md` / `who-fic-ichi.md` for the
//! adapters.
//!
//! This crate parses a file the *user* supplies (downloaded from WHO under
//! WHO's own terms); it does not fetch or embed WHO's classification
//! content.
//!
//! # Format
//!
//! The export is a UTF-8 tab-separated file with a byte-order mark and a
//! header line, one row per entity in the linearization. Individual fields
//! may be bare or Excel/CSV-style double-quoted (`""` escapes an embedded
//! quote); trailing empty columns on a line may be omitted entirely. See
//! `specs/who-fic-linearization.md` in the repository for the full,
//! verified column layout.
//!
//! This crate is exempt from the workspace's usual code-type conventions
//! (`FromStr`/`Display`/canonical-string round-trip): [`LinearizationRow`]
//! is a plain data struct describing one row of an export, not a
//! classification code, so there is no single canonical string form to
//! round-trip through.
//!
//! # Example
//!
//! ```
//! use who_fic_linearization::LinearizationReader;
//!
//! let tsv = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC\n\
//!            http://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"link\"\tTrue\t0\n";
//!
//! let reader = LinearizationReader::from_str(tsv);
//! for result in reader {
//!     let row = result.expect("well-formed row");
//!     assert_eq!(row.code(), Some("1A00"));
//!     assert_eq!(row.title(), "Cholera");
//! }
//! ```

mod error;
mod reader;
mod row;
mod scan;

pub use error::LinearizationError;
pub use reader::LinearizationReader;
pub use row::LinearizationRow;
