//! Optional data-loading adapter (`linearization` feature): builds a lookup
//! from [`IchiCode`] to title from WHO's ICHI "Simplified Linearization
//! Output" export, as parsed row-by-row by
//! [`who_fic_linearization::LinearizationReader`].
//!
//! This module contains no WHO classification content of its own — it only
//! adapts rows the *caller* has already read from a file WHO's export tool
//! produced (see the licensing note in `specs/architecture.md`).
//!
//! # Example
//!
//! ```
//! use who_fic_ichi::linearization::IchiLinearizationIndex;
//! use who_fic_ichi::IchiCode;
//! use who_fic_linearization::LinearizationReader;
//!
//! let tsv = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
//! http://id.who.int/ichi/entity/3\thttp://id.who.int/ichi/release/1/beta/ichi/3\tIAA.BA.BB\t\t\"- - Excision of brain tissue, open approach\"\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n";
//!
//! let reader = LinearizationReader::from_str(tsv);
//! let index = IchiLinearizationIndex::from_rows(reader).unwrap();
//!
//! let code: IchiCode = "IAA.BA.BB".parse().unwrap();
//! assert_eq!(index.title(&code), Some("Excision of brain tissue, open approach"));
//! ```

use std::collections::BTreeMap;
use std::fmt;

use who_fic_linearization::{LinearizationError, LinearizationRow};

use crate::IchiCode;

/// One entry of an [`IchiLinearizationIndex`]: the title and raw
/// `ClassKind` WHO's export recorded for a single [`IchiCode`].
///
/// A plain data struct — construct an index and look entries up through it
/// rather than building this type directly.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IchiClassEntry {
    title: String,
    class_kind: String,
}

impl IchiClassEntry {
    /// The entry's title, e.g. `"Excision of brain tissue, open approach"`.
    ///
    /// Leading `"- "` depth markers are already stripped (see
    /// [`LinearizationRow::title`]).
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The raw `ClassKind` column value for this entry, e.g. `"category"`.
    ///
    /// Every entry retained by [`IchiLinearizationIndex`] came from a row
    /// with `Some(code)`, which in practice means `"category"`, but this is
    /// exposed as the raw string (not a closed enum) since WHO may use
    /// other kinds and this crate does not police the value.
    pub fn class_kind(&self) -> &str {
        &self.class_kind
    }
}

/// Error produced while building an [`IchiLinearizationIndex`] from a
/// [`who_fic_linearization::LinearizationReader`] (or any other iterator of
/// [`LinearizationRow`] results).
///
/// This type wraps [`LinearizationError`] — a malformed *file* (bad UTF-8, a
/// field that doesn't parse) is a genuine error and aborts the whole build.
/// By contrast, an individual well-formed row whose `Code` column doesn't
/// parse as an [`IchiCode`] is *not* a build error: see
/// [`IchiLinearizationIndex::from_rows`] for why those rows are silently
/// skipped instead.
///
/// `#[non_exhaustive]` so a future variant (e.g. duplicate-code detection)
/// can be added without a breaking change.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IchiLinearizationError {
    /// The underlying linearization export could not be read or parsed.
    Read(LinearizationError),
}

impl fmt::Display for IchiLinearizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IchiLinearizationError::Read(source) => {
                write!(f, "failed to read linearization export: {source}")
            }
        }
    }
}

impl std::error::Error for IchiLinearizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IchiLinearizationError::Read(source) => Some(source),
        }
    }
}

impl From<LinearizationError> for IchiLinearizationError {
    fn from(source: LinearizationError) -> Self {
        IchiLinearizationError::Read(source)
    }
}

/// A lookup from [`IchiCode`] to title (and raw class kind), built from a
/// WHO ICHI "Simplified Linearization Output" export.
///
/// Build one with [`from_rows`](Self::from_rows), then look codes up with
/// [`get`](Self::get) or [`title`](Self::title).
///
/// # Hierarchy above each code
///
/// The underlying export also carries `chapter` and `block` rows above each
/// `category` row, whose titles describe the section groupings a code
/// belongs to — a path to eventually deriving real data for
/// [`Section`](crate::Section), which today always returns `None` (see that
/// type's docs). This index does not currently retain those block/chapter
/// titles or the tree structure between them and the codes below; doing so
/// is a natural, self-contained future extension of this module (walk the
/// row stream keeping the most recent `chapter`/`block` title(s) seen at
/// each `DepthInKind`, and attach them to the categories that follow) that
/// was left out of this initial version to keep its surface small.
///
/// # Examples
///
/// ```
/// use who_fic_ichi::linearization::IchiLinearizationIndex;
/// use who_fic_ichi::IchiCode;
/// use who_fic_linearization::LinearizationReader;
///
/// let tsv = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
/// http://id.who.int/ichi/entity/3\thttp://id.who.int/ichi/release/1/beta/ichi/3\tIAA.BA.BB\t\t\"Excision of brain tissue, open approach\"\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n";
///
/// let index = IchiLinearizationIndex::from_rows(LinearizationReader::from_str(tsv)).unwrap();
/// assert_eq!(index.len(), 1);
///
/// let code: IchiCode = "IAA.BA.BB".parse().unwrap();
/// assert_eq!(index.title(&code), Some("Excision of brain tissue, open approach"));
/// assert_eq!(index.get(&code).unwrap().class_kind(), "category");
/// ```
#[derive(Clone, Debug, Default)]
pub struct IchiLinearizationIndex {
    entries: BTreeMap<IchiCode, IchiClassEntry>,
}

impl IchiLinearizationIndex {
    /// Builds an index from an iterator of parsed linearization rows —
    /// typically a [`who_fic_linearization::LinearizationReader`] directly,
    /// since it implements `Iterator<Item = Result<LinearizationRow,
    /// LinearizationError>>`.
    ///
    /// For each row:
    ///
    /// - Rows with `row.code() == None` (`chapter`/`block` rows) are
    ///   skipped.
    /// - Rows with `Some(code)` where `code.parse::<IchiCode>()` succeeds
    ///   are inserted, keyed by the parsed [`IchiCode`], with the row's
    ///   [`title`](LinearizationRow::title) and
    ///   [`class_kind`](LinearizationRow::class_kind).
    /// - Rows with `Some(code)` where parsing *fails* are skipped as well,
    ///   leniently — **this is a deliberate choice, not an oversight.** A
    ///   small number of ICHI Beta-3 entries still marked `(proposed)` use
    ///   a placeholder `??` in place of a real 3-character target axis
    ///   (e.g. `??.BA.BH` instead of a real code like `IAA.BA.BB`), which
    ///   fails [`IchiCode`]'s syntax validation by construction. Failing
    ///   the whole index build over a handful of not-yet-finalized entries
    ///   would make this feature unusable against real WHO exports, so
    ///   those rows are dropped instead. If a row's `Err` here needs to be
    ///   surfaced later, that would be a new, additive
    ///   [`IchiLinearizationError`] variant.
    ///
    /// If the iterator itself yields `Err(LinearizationError)` — meaning
    /// the export file itself was malformed (bad UTF-8, an unparseable
    /// boolean/integer column, …) — that error is propagated immediately
    /// and aborts the build, since it means the source data can't be
    /// trusted at all, unlike a single row's code not parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::linearization::IchiLinearizationIndex;
    /// use who_fic_linearization::LinearizationReader;
    ///
    /// let index = IchiLinearizationIndex::from_rows(LinearizationReader::from_str("")).unwrap();
    /// assert!(index.is_empty());
    /// ```
    pub fn from_rows<I>(rows: I) -> Result<Self, IchiLinearizationError>
    where
        I: Iterator<Item = Result<LinearizationRow, LinearizationError>>,
    {
        let mut entries = BTreeMap::new();
        for row in rows {
            let row = row?;
            let Some(code_str) = row.code() else {
                continue;
            };
            let Ok(code) = code_str.parse::<IchiCode>() else {
                continue;
            };
            entries.insert(
                code,
                IchiClassEntry {
                    title: row.title().to_string(),
                    class_kind: row.class_kind().to_string(),
                },
            );
        }
        Ok(Self { entries })
    }

    /// Returns the title for `code`, if present in the index.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::linearization::IchiLinearizationIndex;
    /// use who_fic_ichi::IchiCode;
    /// use who_fic_linearization::LinearizationReader;
    ///
    /// let index = IchiLinearizationIndex::from_rows(LinearizationReader::from_str("")).unwrap();
    /// let code: IchiCode = "KAB.DB.AD".parse().unwrap();
    /// assert_eq!(index.title(&code), None);
    /// ```
    pub fn title(&self, code: &IchiCode) -> Option<&str> {
        self.entries.get(code).map(|entry| entry.title.as_str())
    }

    /// Returns the full [`IchiClassEntry`] for `code`, if present in the
    /// index.
    pub fn get(&self, code: &IchiCode) -> Option<&IchiClassEntry> {
        self.entries.get(code)
    }

    /// Iterates over every indexed `(code, entry)` pair, in ascending
    /// [`IchiCode`] order.
    pub fn iter(&self) -> impl Iterator<Item = (&IchiCode, &IchiClassEntry)> {
        self.entries.iter()
    }

    /// The number of codes indexed.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use who_fic_linearization::LinearizationReader;

    /// A small hand-written fixture in the "Simplified Linearization
    /// Output" TSV shape (13-column common layout shared by ICF/ICHI
    /// exports): a chapter row, a block row, two real-shaped category
    /// rows, and one `(proposed)` category row using the `??`-placeholder
    /// target axis.
    const FIXTURE: &str = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
        http://id.who.int/ichi/entity/1\thttp://id.who.int/ichi/release/1/beta/ichi/1\t\t\t\"Interventions on the nervous system\"\tchapter\t1\tFalse\tTrue\t\t\tFalse\t2\n\
        http://id.who.int/ichi/entity/2\thttp://id.who.int/ichi/release/1/beta/ichi/2\t\tBlockL1-IAA\t\"- Nervous system procedures\"\tblock\t1\tFalse\tTrue\t\t\tFalse\t2\n\
        http://id.who.int/ichi/entity/3\thttp://id.who.int/ichi/release/1/beta/ichi/3\tIAA.BA.BB\t\t\"- - Excision of brain tissue, open approach\"\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n\
        http://id.who.int/ichi/entity/4\thttp://id.who.int/ichi/release/1/beta/ichi/4\tIAA.BA.BC\t\t\"- - Excision of brain tissue, endoscopic approach\"\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n\
        http://id.who.int/ichi/entity/5\thttp://id.who.int/ichi/release/1/beta/ichi/5\t??.BA.BH\t\t\"- - Excision of brain tissue, other approach (proposed)\"\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n";

    fn build_index() -> IchiLinearizationIndex {
        let reader = LinearizationReader::from_str(FIXTURE);
        IchiLinearizationIndex::from_rows(reader).expect("fixture is well-formed")
    }

    #[test]
    fn indexes_valid_category_rows() {
        let index = build_index();
        assert_eq!(index.len(), 2);

        let bb: IchiCode = "IAA.BA.BB".parse().unwrap();
        assert_eq!(
            index.title(&bb),
            Some("Excision of brain tissue, open approach")
        );
        assert_eq!(index.get(&bb).unwrap().class_kind(), "category");

        let bc: IchiCode = "IAA.BA.BC".parse().unwrap();
        assert_eq!(
            index.title(&bc),
            Some("Excision of brain tissue, endoscopic approach")
        );
    }

    #[test]
    fn excludes_chapter_and_block_rows() {
        let index = build_index();
        // Neither the chapter nor the block row has a `Code`, so neither
        // could have contributed a key to the index; there is no code to
        // look them up by. We confirm indirectly: only the two well-formed
        // category rows made it in.
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn skips_placeholder_target_rows_leniently() {
        let index = build_index();
        // The `??`-placeholder proposed entry's `Code` fails `IchiCode`
        // parsing (target segment is 2 chars, not 3) and must be skipped,
        // not fail the whole build.
        assert!("??.BA.BH".parse::<IchiCode>().is_err());
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn iter_visits_every_entry_in_code_order() {
        let index = build_index();
        let codes: Vec<String> = index.iter().map(|(code, _)| code.to_string()).collect();
        assert_eq!(
            codes,
            vec!["IAA.BA.BB".to_string(), "IAA.BA.BC".to_string()]
        );
    }

    #[test]
    fn empty_input_yields_empty_index() {
        let index = IchiLinearizationIndex::from_rows(LinearizationReader::from_str("")).unwrap();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert_eq!(index.title(&"IAA.BA.BB".parse().unwrap()), None);
    }

    #[test]
    fn propagates_malformed_file_errors() {
        // A row whose DepthInKind column is not a valid integer is a
        // genuine file-level error, and must abort the whole build rather
        // than being silently skipped.
        let bad = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
            http://id.who.int/ichi/entity/1\thttp://id.who.int/ichi/release/1/beta/ichi/1\tIAA.BA.BB\t\t\"Title\"\tcategory\tnot-a-number\tFalse\tTrue\t\t\tTrue\t0\n";
        let err =
            IchiLinearizationIndex::from_rows(LinearizationReader::from_str(bad)).unwrap_err();
        assert!(matches!(err, IchiLinearizationError::Read(_)));
        assert!(
            err.to_string()
                .contains("failed to read linearization export")
        );
    }

    #[test]
    fn class_entry_accessors() {
        let index = build_index();
        let code: IchiCode = "IAA.BA.BB".parse().unwrap();
        let entry = index.get(&code).unwrap();
        assert_eq!(entry.title(), "Excision of brain tissue, open approach");
        assert_eq!(entry.class_kind(), "category");
    }
}
