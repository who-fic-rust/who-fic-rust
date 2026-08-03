use std::io::{self, BufRead, BufReader, Cursor, Read};

use crate::error::LinearizationError;
use crate::row::{self, Layout, LinearizationRow};

/// A byte-order mark, which WHO's export tool writes at the start of the
/// file.
const BOM: char = '\u{feff}';

/// Strips a leading UTF-8 byte-order mark, if present.
fn strip_bom(line: &str) -> &str {
    line.strip_prefix(BOM).unwrap_or(line)
}

/// A streaming row-by-row reader for WHO's Simplified Linearization Output
/// TSV export format.
///
/// `LinearizationReader` is an [`Iterator`] of
/// `Result<`[`LinearizationRow`]`, `[`LinearizationError`]`>`; it reads one
/// line at a time rather than buffering the whole file, so a multi-megabyte
/// MMS export does not need to fit in memory all at once.
///
/// The header line is read and consumed automatically by the first call to
/// [`next`](Iterator::next) (it is never yielded as a row). The reader
/// inspects the header's column names to tell the 13-column layout shared by
/// ICF/ICHI exports apart from the 18-column MMS layout (13 common columns
/// plus `Primary tabulation` and `Grouping1`..`Grouping5`), by checking for
/// a `Grouping1` column — so [`LinearizationRow::groupings`] is populated
/// correctly either way without the caller having to say which
/// classification the file came from.
///
/// # Examples
///
/// ```
/// use who_fic_linearization::LinearizationReader;
///
/// let tsv = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC\nhttp://id.who.int/icd/entity/1435254666\thttp://id.who.int/icd/release/11/beta/mms/1435254666\t\t\t\"Certain infectious or parasitic diseases\"\tchapter\t1\tFalse\tTrue\t01\t\"link\"\tFalse\t22\n";
/// let reader = LinearizationReader::from_str(tsv);
/// let rows: Vec<_> = reader.collect::<Result<_, _>>().unwrap();
/// assert_eq!(rows.len(), 1);
/// assert_eq!(rows[0].class_kind(), "chapter");
/// assert_eq!(rows[0].foundation_uri(), Some("http://id.who.int/icd/entity/1435254666"));
/// ```
pub struct LinearizationReader<R> {
    reader: BufReader<R>,
    line_no: usize,
    layout: Option<Layout>,
    done: bool,
}

impl<R: Read> LinearizationReader<R> {
    /// Builds a reader from any [`Read`] source, e.g. an open [`std::fs::File`].
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_linearization::LinearizationReader;
    /// use std::io::Cursor;
    ///
    /// let tsv = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC\n";
    /// let reader = LinearizationReader::from_reader(Cursor::new(tsv.as_bytes()));
    /// assert_eq!(reader.count(), 0);
    /// ```
    pub fn from_reader(inner: R) -> Self {
        Self {
            reader: BufReader::new(inner),
            line_no: 0,
            layout: None,
            done: false,
        }
    }

    /// Reads one line (without its trailing `\n`/`\r\n`), advancing
    /// `line_no`. Returns `Ok(None)` at end of input.
    fn read_line(&mut self) -> io::Result<Option<String>> {
        let mut buf = String::new();
        let bytes_read = self.reader.read_line(&mut buf)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        self.line_no += 1;
        while buf.ends_with('\n') || buf.ends_with('\r') {
            buf.pop();
        }
        Ok(Some(buf))
    }
}

impl LinearizationReader<Cursor<Vec<u8>>> {
    /// Builds a reader over an in-memory string, e.g. a fixture in a test.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_linearization::LinearizationReader;
    ///
    /// let reader = LinearizationReader::from_str("Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n");
    /// assert_eq!(reader.count(), 0);
    /// ```
    // Named to match `specs/who-fic-linearization.md`'s documented
    // constructor pair (`from_str`/`from_reader`). This is deliberately an
    // inherent method, not `std::str::FromStr`: construction here never
    // fails (parse errors surface lazily, per row, from the iterator), so
    // the trait's `Result`-returning signature does not fit.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &str) -> Self {
        Self::from_reader(Cursor::new(input.as_bytes().to_vec()))
    }
}

impl<R: Read> Iterator for LinearizationReader<R> {
    type Item = Result<LinearizationRow, LinearizationError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if self.layout.is_none() {
            match self.read_line() {
                Ok(Some(header)) => {
                    self.layout = Some(Layout::detect(strip_bom(&header)));
                }
                Ok(None) => {
                    self.done = true;
                    return None;
                }
                Err(source) => {
                    self.done = true;
                    return Some(Err(LinearizationError::Io {
                        line: self.line_no.max(1),
                        message: source.to_string(),
                    }));
                }
            }
        }

        loop {
            match self.read_line() {
                Ok(Some(line)) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let layout = self.layout.expect("layout is set before any row is read");
                    return Some(row::parse_row(&line, layout, self.line_no));
                }
                Ok(None) => {
                    self.done = true;
                    return None;
                }
                Err(source) => {
                    self.done = true;
                    return Some(Err(LinearizationError::Io {
                        line: self.line_no.max(1),
                        message: source.to_string(),
                    }));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ICF_HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC\n";

    const MMS_HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tPrimary tabulation\tGrouping1\tGrouping2\tGrouping3\tGrouping4\tGrouping5\tVersion:2026 Apr 15 - 14:33 UTC\n";

    #[test]
    fn empty_input_yields_no_rows() {
        let mut reader = LinearizationReader::from_str("");
        assert!(reader.next().is_none());
    }

    #[test]
    fn header_only_yields_no_rows() {
        let mut reader = LinearizationReader::from_str(ICF_HEADER);
        assert!(reader.next().is_none());
    }

    #[test]
    fn strips_bom_from_header() {
        let input = format!(
            "\u{feff}{ICF_HEADER}http://id.who.int/icd/entity/1\thttp://id.who.int/icd/release/11/beta/icf/1\tb110\t\t\"Title\"\tcategory\t1\tFalse\tTrue\t\t\"link\"\tTrue\t0\n"
        );
        let mut reader = LinearizationReader::from_str(&input);
        let row = reader.next().unwrap().unwrap();
        assert_eq!(row.code(), Some("b110"));
    }

    #[test]
    fn detects_mms_layout_and_parses_groupings() {
        let input = format!(
            "{MMS_HEADER}http://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"link\"\tTrue\t0\tTrue\tBlockL1-1A0\tBlockL2-1A0\t\t\t\n"
        );
        let mut reader = LinearizationReader::from_str(&input);
        let row = reader.next().unwrap().unwrap();
        assert_eq!(
            row.groupings(),
            &["BlockL1-1A0".to_string(), "BlockL2-1A0".to_string()]
        );
        assert_eq!(row.primary_tabulation(), Some(true));
    }

    #[test]
    fn detects_common_layout_without_groupings() {
        let input = format!(
            "{ICF_HEADER}http://id.who.int/icd/entity/1\thttp://id.who.int/icd/release/11/beta/icf/1\tb110\t\t\"Title\"\tcategory\t1\tFalse\tTrue\t\t\"link\"\tTrue\t0\n"
        );
        let mut reader = LinearizationReader::from_str(&input);
        let row = reader.next().unwrap().unwrap();
        assert_eq!(row.groupings(), &[] as &[String]);
        assert_eq!(row.primary_tabulation(), None);
    }

    #[test]
    fn line_numbers_are_1_based_and_count_the_header() {
        let input = format!(
            "{ICF_HEADER}http://id.who.int/icd/entity/1\thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Title\"\tcategory\tnot-a-number\tFalse\tTrue\t\t\"link\"\tTrue\t0\n"
        );
        let mut reader = LinearizationReader::from_str(&input);
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line(), 2);
    }

    #[test]
    fn skips_blank_lines_between_rows() {
        let input = format!(
            "{ICF_HEADER}\nhttp://id.who.int/icd/entity/1\thttp://id.who.int/icd/release/11/beta/icf/1\tb110\t\t\"Title\"\tcategory\t1\tFalse\tTrue\t\t\"link\"\tTrue\t0\n\n"
        );
        let mut reader = LinearizationReader::from_str(&input);
        let row = reader.next().unwrap().unwrap();
        assert_eq!(row.code(), Some("b110"));
        assert!(reader.next().is_none());
    }
}
