//! Adapts [`who_fic_linearization::LinearizationRow`]s from WHO's ICF
//! "Simplified Linearization Output" export into a lookup from [`IcfCode`]
//! to title.
//!
//! The user supplies the exported file (e.g. via
//! [`LinearizationReader`](who_fic_linearization::LinearizationReader)); this
//! crate never bundles WHO content (see `specs/architecture.md`).
//!
//! # Example
//!
//! ```
//! use std::str::FromStr;
//! use who_fic_icf::linearization::IcfLinearizationIndex;
//! use who_fic_icf::IcfCode;
//! use who_fic_linearization::LinearizationReader;
//!
//! let tsv = "\
//! Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
//! \thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Mental functions\"\tchapter\t1\tFalse\tTrue\t\t\tFalse\t1\n\
//! \thttp://id.who.int/icd/release/11/beta/icf/2\tb110\t\t\"Consciousness functions\"\tcategory\t2\tFalse\tTrue\t\t\tTrue\t0\n";
//!
//! let reader = LinearizationReader::from_str(tsv);
//! let index = IcfLinearizationIndex::from_rows(reader).unwrap();
//!
//! let code = IcfCode::from_str("b110").unwrap();
//! assert_eq!(index.title(&code), Some("Consciousness functions"));
//! ```

use std::collections::BTreeMap;
use std::collections::btree_map;
use std::fmt;
use std::str::FromStr;

use who_fic_linearization::{LinearizationError, LinearizationRow};

use crate::IcfCode;

/// One indexed entry from an ICF linearization export: a code, its title,
/// and the raw `ClassKind` of the row it came from.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::linearization::IcfLinearizationIndex;
/// use who_fic_icf::IcfCode;
/// use who_fic_linearization::LinearizationReader;
///
/// let tsv = "\
/// Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
/// \thttp://id.who.int/icd/release/11/beta/icf/1\tb110\t\t\"Consciousness functions\"\tcategory\t2\tFalse\tTrue\t\t\tTrue\t0\n";
/// let index = IcfLinearizationIndex::from_rows(LinearizationReader::from_str(tsv)).unwrap();
///
/// let entry = index.get(&IcfCode::from_str("b110").unwrap()).unwrap();
/// assert_eq!(entry.code(), &IcfCode::from_str("b110").unwrap());
/// assert_eq!(entry.title(), "Consciousness functions");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IcfClassEntry {
    code: IcfCode,
    title: String,
    class_kind: String,
}

impl IcfClassEntry {
    /// The entry's code.
    pub fn code(&self) -> &IcfCode {
        &self.code
    }

    /// The entry's title, as given by the export's `Title` column (with
    /// leading depth-dash markers already stripped by
    /// [`who_fic_linearization`]).
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The raw `ClassKind` column value for the row this entry came from
    /// (`category` for every entry reachable through
    /// [`IcfLinearizationIndex`], since only `category` rows carry a code).
    pub fn class_kind(&self) -> &str {
        &self.class_kind
    }
}

/// A lookup from [`IcfCode`] to title, built from a WHO ICF "Simplified
/// Linearization Output" export.
///
/// Construct via [`from_rows`](Self::from_rows), feeding it the rows from a
/// [`who_fic_linearization::LinearizationReader`]. Only `category` rows (the
/// ones with a `Code` column) are indexed; `chapter` and `block` rows have no
/// code and are skipped. Rows whose `Code` fails to parse as an [`IcfCode`]
/// are also skipped rather than failing the whole build (see
/// [`from_rows`](Self::from_rows) for why).
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::linearization::IcfLinearizationIndex;
/// use who_fic_icf::IcfCode;
/// use who_fic_linearization::LinearizationReader;
///
/// let tsv = "\
/// Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
/// \thttp://id.who.int/icd/release/11/beta/icf/1\tb110\t\t\"Consciousness functions\"\tcategory\t2\tFalse\tTrue\t\t\tTrue\t0\n";
///
/// let reader = LinearizationReader::from_str(tsv);
/// let index = IcfLinearizationIndex::from_rows(reader).unwrap();
/// assert_eq!(index.len(), 1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct IcfLinearizationIndex {
    entries: BTreeMap<IcfCode, IcfClassEntry>,
}

impl IcfLinearizationIndex {
    /// Builds an index from an iterator of linearization rows, e.g. a
    /// [`who_fic_linearization::LinearizationReader`] itself.
    ///
    /// For each row:
    ///
    /// - if [`row.code()`](LinearizationRow::code) is `Some(s)` and `s`
    ///   parses as an [`IcfCode`], the code is inserted into the index with
    ///   its title from [`row.title()`](LinearizationRow::title);
    /// - if `row.code()` is `Some(s)` but `s` does *not* parse as an
    ///   [`IcfCode`], the row is silently skipped rather than failing the
    ///   whole build. Real-world WHO exports are known to contain
    ///   proposed/placeholder entries that don't conform to the finalized
    ///   code grammar (see `specs/who-fic-icf.md`), and treating every such
    ///   row as fatal would make the index unusable against real exports;
    /// - if `row.code()` is `None` (`chapter`/`block` rows), the row is
    ///   skipped from this code-keyed index.
    ///
    /// This method takes `impl Iterator<Item = Result<LinearizationRow,
    /// LinearizationError>>` -- exactly what a
    /// [`who_fic_linearization::LinearizationReader`] yields -- so a reader
    /// can be passed straight in. A genuine `Err` from the reader (the file
    /// itself is malformed) is propagated immediately as
    /// [`IcfLinearizationError::Reader`]; that is a stricter kind of failure
    /// than a `Code` column that simply doesn't parse as an [`IcfCode`],
    /// which is not fatal (see above).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::linearization::IcfLinearizationIndex;
    /// use who_fic_icf::IcfCode;
    /// use who_fic_linearization::LinearizationReader;
    ///
    /// let tsv = "\
    /// Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
    /// \thttp://id.who.int/icd/release/11/beta/icf/1\tb110\t\t\"Consciousness functions\"\tcategory\t2\tFalse\tTrue\t\t\tTrue\t0\n";
    ///
    /// let index = IcfLinearizationIndex::from_rows(LinearizationReader::from_str(tsv)).unwrap();
    /// let code = IcfCode::from_str("b110").unwrap();
    /// assert_eq!(index.title(&code), Some("Consciousness functions"));
    /// ```
    pub fn from_rows<I>(rows: I) -> Result<Self, IcfLinearizationError>
    where
        I: Iterator<Item = Result<LinearizationRow, LinearizationError>>,
    {
        let mut entries = BTreeMap::new();
        for row in rows {
            let row = row?;
            let Some(raw_code) = row.code() else {
                // chapter/block rows: no code, nothing to index here.
                continue;
            };
            let Ok(code) = IcfCode::from_str(raw_code) else {
                // Lenient skip: see the doc comment above for why a
                // non-parsing `Code` column is not treated as fatal.
                continue;
            };
            entries.insert(
                code.clone(),
                IcfClassEntry {
                    code,
                    title: row.title().to_string(),
                    class_kind: row.class_kind().to_string(),
                },
            );
        }
        Ok(Self { entries })
    }

    /// The title of `code`, if it was indexed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::linearization::IcfLinearizationIndex;
    /// use who_fic_icf::IcfCode;
    /// use who_fic_linearization::LinearizationReader;
    ///
    /// let tsv = "\
    /// Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
    /// \thttp://id.who.int/icd/release/11/beta/icf/1\tb110\t\t\"Consciousness functions\"\tcategory\t2\tFalse\tTrue\t\t\tTrue\t0\n";
    /// let reader = LinearizationReader::from_str(tsv);
    /// let index = IcfLinearizationIndex::from_rows(reader).unwrap();
    ///
    /// let indexed = IcfCode::from_str("b110").unwrap();
    /// assert_eq!(index.title(&indexed), Some("Consciousness functions"));
    ///
    /// let unindexed = IcfCode::from_str("b280").unwrap();
    /// assert_eq!(index.title(&unindexed), None);
    /// ```
    pub fn title(&self, code: &IcfCode) -> Option<&str> {
        self.entries.get(code).map(|entry| entry.title())
    }

    /// The full entry for `code`, if it was indexed.
    pub fn get(&self, code: &IcfCode) -> Option<&IcfClassEntry> {
        self.entries.get(code)
    }

    /// Iterates over all indexed entries, in ascending code order.
    pub fn iter(&self) -> impl Iterator<Item = &IcfClassEntry> {
        self.entries.values()
    }

    /// The number of indexed codes.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<'a> IntoIterator for &'a IcfLinearizationIndex {
    type Item = &'a IcfClassEntry;
    type IntoIter = btree_map::Values<'a, IcfCode, IcfClassEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.values()
    }
}

/// Error building an [`IcfLinearizationIndex`].
///
/// The only source of failure is the underlying linearization reader itself
/// reporting a malformed file; an [`IcfCode`] that fails to parse from a
/// row's `Code` column is *not* an error here (see
/// [`IcfLinearizationIndex::from_rows`]).
///
/// `#[non_exhaustive]` so new variants can be added without a breaking
/// change.
///
/// # Examples
///
/// ```
/// use who_fic_icf::linearization::IcfLinearizationIndex;
/// use who_fic_linearization::{LinearizationError, LinearizationReader};
///
/// // An unterminated quoted field is a genuine, fatal error: the file
/// // itself is malformed, unlike a merely non-parsing `Code` value.
/// let tsv = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Unterminated\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n";
/// let reader = LinearizationReader::from_str(tsv);
/// let err = IcfLinearizationIndex::from_rows(reader).unwrap_err();
/// assert!(matches!(
///     err,
///     who_fic_icf::linearization::IcfLinearizationError::Reader(_)
/// ));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IcfLinearizationError {
    /// The underlying [`LinearizationReader`](who_fic_linearization::LinearizationReader)
    /// reported an error reading or parsing the export file.
    Reader(LinearizationError),
}

impl From<LinearizationError> for IcfLinearizationError {
    fn from(source: LinearizationError) -> Self {
        IcfLinearizationError::Reader(source)
    }
}

impl fmt::Display for IcfLinearizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcfLinearizationError::Reader(source) => {
                write!(f, "linearization reader error: {source}")
            }
        }
    }
}

impl std::error::Error for IcfLinearizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IcfLinearizationError::Reader(source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use who_fic_linearization::LinearizationReader;

    const HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n";

    /// A tiny fixture: a chapter row (no code), two valid category rows, and
    /// one category row whose `Code` doesn't parse as an `IcfCode` (exercises
    /// the lenient-skip path).
    fn fixture_tsv() -> String {
        format!(
            "{HEADER}\
             \thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Mental functions\"\tchapter\t1\tFalse\tTrue\t\t\tFalse\t2\n\
             \thttp://id.who.int/icd/release/11/beta/icf/2\tb110\t\t\"Consciousness functions\"\tcategory\t2\tFalse\tTrue\t\t\tFalse\t1\n\
             \thttp://id.who.int/icd/release/11/beta/icf/3\tb1100\t\t\"State of consciousness\"\tcategory\t3\tFalse\tTrue\t\t\tTrue\t0\n\
             \thttp://id.who.int/icd/release/11/beta/icf/4\tb1xyz\t\t\"Proposed placeholder entry\"\tcategory\t2\tFalse\tTrue\t\t\tTrue\t0\n"
        )
    }

    fn fixture_index() -> IcfLinearizationIndex {
        let reader = LinearizationReader::from_str(&fixture_tsv());
        IcfLinearizationIndex::from_rows(reader).unwrap()
    }

    #[test]
    fn indexes_valid_category_codes() {
        let index = fixture_index();
        let b110 = IcfCode::from_str("b110").unwrap();
        let b1100 = IcfCode::from_str("b1100").unwrap();
        assert_eq!(index.title(&b110), Some("Consciousness functions"));
        assert_eq!(index.title(&b1100), Some("State of consciousness"));
        assert_eq!(index.get(&b110).unwrap().class_kind(), "category");
        assert_eq!(index.get(&b110).unwrap().code(), &b110);
    }

    #[test]
    fn chapter_row_is_excluded_from_the_code_index() {
        let index = fixture_index();
        // The chapter row has no `Code` column at all, so there is no
        // `IcfCode` to look it up by; confirm it didn't leak in as some
        // other entry and that the index size matches only the two valid
        // category rows.
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn unparseable_code_is_skipped_leniently() {
        let index = fixture_index();
        // "b1xyz" fails IcfCode::from_str (non-digit characters), and must
        // not have been inserted, nor should building the index have
        // failed.
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn unindexed_code_returns_none() {
        let index = fixture_index();
        let unindexed = IcfCode::from_str("s730").unwrap();
        assert_eq!(index.title(&unindexed), None);
        assert_eq!(index.get(&unindexed), None);
    }

    #[test]
    fn iter_visits_all_entries_in_ascending_code_order() {
        let index = fixture_index();
        let codes: Vec<&str> = index.iter().map(|entry| entry.code().as_str()).collect();
        assert_eq!(codes, vec!["b110", "b1100"]);
    }

    #[test]
    fn into_iter_on_reference_matches_iter() {
        let index = fixture_index();
        let codes: Vec<&str> = (&index)
            .into_iter()
            .map(|entry| entry.code().as_str())
            .collect();
        assert_eq!(codes, vec!["b110", "b1100"]);
    }

    #[test]
    fn is_empty_reports_correctly() {
        let empty = IcfLinearizationIndex::from_rows(std::iter::empty()).unwrap();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let index = fixture_index();
        assert!(!index.is_empty());
    }

    #[test]
    fn reader_error_propagates_as_icf_linearization_error() {
        // An unterminated quoted field is a genuine, fatal parse error from
        // the underlying reader, not a lenient-skip case.
        let bad_tsv = format!(
            "{HEADER}\thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Unterminated\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n"
        );
        let reader = LinearizationReader::from_str(&bad_tsv);
        let err = IcfLinearizationIndex::from_rows(reader).unwrap_err();
        assert!(matches!(err, IcfLinearizationError::Reader(_)));
    }

    #[test]
    fn error_display_includes_source_message() {
        let source = LinearizationError::UnterminatedQuotedField { line: 2 };
        let err = IcfLinearizationError::from(source);
        assert!(err.to_string().contains("linearization reader error"));
    }
}
