use crate::error::LinearizationError;
use crate::scan::{self, ScanErrorKind};

/// Which of the two known Simplified Linearization Output header shapes a
/// [`crate::LinearizationReader`] detected.
///
/// MMS (ICD-11) exports carry 5 extra `Primary tabulation` /
/// `Grouping1`..`Grouping5` columns that ICF and ICHI exports do not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Layout {
    /// The 13-column layout shared by ICF and ICHI exports (and the base of
    /// the MMS layout).
    Common,
    /// The 18-column layout used by MMS exports: the 13 common columns plus
    /// `Primary tabulation` and `Grouping1`..`Grouping5`.
    Mms,
}

impl Layout {
    /// Detects the layout from an already BOM-stripped header line.
    ///
    /// Detection is based on column *names* rather than column *count*,
    /// since column count alone is fragile against trailing short lines;
    /// the header line itself is always well-formed in practice.
    pub(crate) fn detect(header_line: &str) -> Self {
        let has_grouping_column = header_line
            .split('\t')
            .any(|column| column.trim().trim_matches('"') == "Grouping1");
        if has_grouping_column {
            Layout::Mms
        } else {
            Layout::Common
        }
    }
}

/// One data row of a WHO Simplified Linearization Output export: a single
/// entity (chapter, block, or category) in a linearization of ICD-11 (MMS),
/// ICF, or ICHI.
///
/// Construct rows by iterating a [`crate::LinearizationReader`]; there is no
/// public constructor, since a `LinearizationRow` only ever comes from
/// parsing one line of a real export.
///
/// # Examples
///
/// ```
/// use who_fic_linearization::LinearizationReader;
///
/// let tsv = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC\nhttp://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"link\"\tTrue\t0\n";
/// let mut reader = LinearizationReader::from_str(tsv);
/// let row = reader.next().unwrap().unwrap();
/// assert_eq!(row.code(), Some("1A00"));
/// assert_eq!(row.title(), "Cholera");
/// assert_eq!(row.class_kind(), "category");
/// assert!(!row.is_residual());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinearizationRow {
    foundation_uri: Option<String>,
    linearization_uri: String,
    code: Option<String>,
    block_id: Option<String>,
    title: String,
    class_kind: String,
    depth_in_kind: u32,
    is_residual: bool,
    primary_location: Option<bool>,
    chapter_no: Option<String>,
    browser_link: Option<String>,
    is_leaf: bool,
    no_of_non_residual_children: u32,
    primary_tabulation: Option<bool>,
    groupings: Vec<String>,
}

impl LinearizationRow {
    /// The `Foundation URI` column, e.g.
    /// `http://id.who.int/icd/entity/257068234`.
    ///
    /// `None` for residual (`.Y`/`.Z`, `/other`, `/unspecified`) rows, which
    /// have no foundation-layer entity of their own.
    pub fn foundation_uri(&self) -> Option<&str> {
        self.foundation_uri.as_deref()
    }

    /// The `Linearization (release) URI` column. Always present.
    pub fn linearization_uri(&self) -> &str {
        &self.linearization_uri
    }

    /// The `Code` column: the classification's code for `category` rows.
    ///
    /// `None` for `chapter` and `block` rows, which are not themselves
    /// codes.
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// The `BlockId` column, e.g. `BlockL1-1A0` for `block` rows.
    ///
    /// `None` for `chapter` and `category` rows.
    pub fn block_id(&self) -> Option<&str> {
        self.block_id.as_deref()
    }

    /// The `Title` column, with leading `"- "` depth markers stripped (the
    /// depth is already available numerically via
    /// [`depth_in_kind`](Self::depth_in_kind)).
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The `ClassKind` column: `chapter`, `block`, or `category` in
    /// practice, but exposed as a string rather than a closed enum since
    /// WHO may add kinds.
    pub fn class_kind(&self) -> &str {
        &self.class_kind
    }

    /// The `DepthInKind` column: this row's depth within its `ClassKind`.
    pub fn depth_in_kind(&self) -> u32 {
        self.depth_in_kind
    }

    /// The `IsResidual` column: whether this row is a residual
    /// (`.Y`/`.Z`, "other"/"unspecified") category.
    pub fn is_residual(&self) -> bool {
        self.is_residual
    }

    /// The `PrimaryLocation` column.
    ///
    /// `None` when the column was empty.
    pub fn primary_location(&self) -> Option<bool> {
        self.primary_location
    }

    /// The `ChapterNo` column, e.g. `01`.
    ///
    /// `None` when empty, which is the case for every ICF and ICHI row seen
    /// in practice.
    pub fn chapter_no(&self) -> Option<&str> {
        self.chapter_no.as_deref()
    }

    /// The `BrowserLink` column: an Excel `=hyperlink(...)` formula string,
    /// exposed exactly as written in the export (this crate does not parse
    /// it).
    ///
    /// `None` when empty.
    pub fn browser_link(&self) -> Option<&str> {
        self.browser_link.as_deref()
    }

    /// The `isLeaf` column: whether this row has no children in the
    /// linearization.
    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    /// The `noOfNonResidualChildren` column.
    pub fn no_of_non_residual_children(&self) -> u32 {
        self.no_of_non_residual_children
    }

    /// The `Primary tabulation` column. MMS exports only.
    ///
    /// `None` when the source file had no `Primary tabulation` column
    /// (ICF/ICHI layout), or when the column was present but empty.
    pub fn primary_tabulation(&self) -> Option<bool> {
        self.primary_tabulation
    }

    /// The non-empty values among `Grouping1`..`Grouping5`, in column
    /// order.
    ///
    /// Empty (ICF/ICHI layout, which has no `Grouping` columns at all, or an
    /// MMS row that used fewer than 5 groupings).
    pub fn groupings(&self) -> &[String] {
        &self.groupings
    }
}

/// Strips leading repeated `"- "` (dash space) depth markers from a raw
/// `Title` value.
fn strip_depth_markers(raw: &str) -> &str {
    let mut s = raw;
    while let Some(rest) = s.strip_prefix("- ") {
        s = rest;
    }
    s
}

/// Returns `None` for an empty (after trimming) field, `Some` otherwise.
fn non_empty(field: &str) -> Option<String> {
    if field.trim().is_empty() {
        None
    } else {
        Some(field.to_string())
    }
}

/// Parses a `True`/`False` field, defaulting a missing/empty field to
/// `false`.
fn parse_bool(field: &str, name: &'static str, line: usize) -> Result<bool, LinearizationError> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        Ok(false)
    } else if trimmed.eq_ignore_ascii_case("true") {
        Ok(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        Ok(false)
    } else {
        Err(LinearizationError::InvalidBoolean {
            line,
            field: name,
            found: field.to_string(),
        })
    }
}

/// Parses an optional `True`/`False`/empty field.
fn parse_opt_bool(
    field: &str,
    name: &'static str,
    line: usize,
) -> Result<Option<bool>, LinearizationError> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else if trimmed.eq_ignore_ascii_case("true") {
        Ok(Some(true))
    } else if trimmed.eq_ignore_ascii_case("false") {
        Ok(Some(false))
    } else {
        Err(LinearizationError::InvalidBoolean {
            line,
            field: name,
            found: field.to_string(),
        })
    }
}

/// Parses an unsigned-integer field, defaulting a missing/empty field to 0.
fn parse_u32(field: &str, name: &'static str, line: usize) -> Result<u32, LinearizationError> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    trimmed
        .parse::<u32>()
        .map_err(|_| LinearizationError::InvalidInteger {
            line,
            field: name,
            found: field.to_string(),
        })
}

/// Returns the field at `index`, or `""` if the line was short and omitted
/// it (and any columns after it).
fn field_at(fields: &[String], index: usize) -> &str {
    fields.get(index).map(String::as_str).unwrap_or("")
}

/// Parses one non-header, non-blank line into a [`LinearizationRow`].
pub(crate) fn parse_row(
    line: &str,
    layout: Layout,
    line_no: usize,
) -> Result<LinearizationRow, LinearizationError> {
    let fields = scan::split_tab_csv_line(line).map_err(|kind| match kind {
        ScanErrorKind::UnterminatedQuote => {
            LinearizationError::UnterminatedQuotedField { line: line_no }
        }
        ScanErrorKind::TrailingDataAfterQuotedField => {
            LinearizationError::TrailingDataAfterQuotedField { line: line_no }
        }
    })?;

    let foundation_uri = non_empty(field_at(&fields, 0));
    let linearization_uri = field_at(&fields, 1).to_string();
    let code = non_empty(field_at(&fields, 2));
    let block_id = non_empty(field_at(&fields, 3));
    let title = strip_depth_markers(field_at(&fields, 4)).to_string();
    let class_kind = field_at(&fields, 5).to_string();
    let depth_in_kind = parse_u32(field_at(&fields, 6), "DepthInKind", line_no)?;
    let is_residual = parse_bool(field_at(&fields, 7), "IsResidual", line_no)?;
    let primary_location = parse_opt_bool(field_at(&fields, 8), "PrimaryLocation", line_no)?;
    let chapter_no = non_empty(field_at(&fields, 9));
    let browser_link = non_empty(field_at(&fields, 10));
    let is_leaf = parse_bool(field_at(&fields, 11), "isLeaf", line_no)?;
    let no_of_non_residual_children =
        parse_u32(field_at(&fields, 12), "noOfNonResidualChildren", line_no)?;

    let (primary_tabulation, groupings) = if layout == Layout::Mms {
        let primary_tabulation =
            parse_opt_bool(field_at(&fields, 13), "Primary tabulation", line_no)?;
        let groupings = (14..19)
            .map(|index| field_at(&fields, index))
            .filter(|field| !field.trim().is_empty())
            .map(str::to_string)
            .collect();
        (primary_tabulation, groupings)
    } else {
        (None, Vec::new())
    };

    Ok(LinearizationRow {
        foundation_uri,
        linearization_uri,
        code,
        block_id,
        title,
        class_kind,
        depth_in_kind,
        is_residual,
        primary_location,
        chapter_no,
        browser_link,
        is_leaf,
        no_of_non_residual_children,
        primary_tabulation,
        groupings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_repeated_depth_markers() {
        assert_eq!(strip_depth_markers("- - - Cholera"), "Cholera");
        assert_eq!(strip_depth_markers("ICF Category"), "ICF Category");
        assert_eq!(
            strip_depth_markers("- - - - Consciousness"),
            "Consciousness"
        );
    }

    #[test]
    fn layout_detects_mms_from_grouping_column() {
        let header = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tPrimary tabulation\tGrouping1\tGrouping2\tGrouping3\tGrouping4\tGrouping5\tVersion:2026 Apr 15 - 14:33 UTC";
        assert_eq!(Layout::detect(header), Layout::Mms);
    }

    #[test]
    fn layout_detects_common_without_grouping_column() {
        let header = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC";
        assert_eq!(Layout::detect(header), Layout::Common);
    }

    #[test]
    fn parses_short_line_with_defaults() {
        // Only the first 5 columns present; everything after defaults.
        let row = parse_row(
            "http://id.who.int/icd/entity/1\thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Title\"",
            Layout::Common,
            2,
        )
        .unwrap();
        assert_eq!(row.class_kind(), "");
        assert_eq!(row.depth_in_kind(), 0);
        assert!(!row.is_residual());
        assert_eq!(row.primary_location(), None);
        assert_eq!(row.chapter_no(), None);
        assert!(!row.is_leaf());
        assert_eq!(row.no_of_non_residual_children(), 0);
        assert_eq!(row.groupings(), &[] as &[String]);
    }

    #[test]
    fn invalid_boolean_reports_line_and_field() {
        let err = parse_row(
            "\thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Title\"\tcategory\t1\tmaybe\tTrue\t\t\t\ttrue\t0",
            Layout::Common,
            5,
        )
        .unwrap_err();
        assert_eq!(
            err,
            LinearizationError::InvalidBoolean {
                line: 5,
                field: "IsResidual",
                found: "maybe".to_string(),
            }
        );
    }

    #[test]
    fn invalid_integer_reports_line_and_field() {
        let err = parse_row(
            "\thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Title\"\tcategory\tabc\tFalse\tTrue\t\t\t\ttrue\t0",
            Layout::Common,
            7,
        )
        .unwrap_err();
        assert_eq!(
            err,
            LinearizationError::InvalidInteger {
                line: 7,
                field: "DepthInKind",
                found: "abc".to_string(),
            }
        );
    }
}
