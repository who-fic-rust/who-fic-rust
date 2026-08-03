use std::fmt;

/// An error produced while reading or parsing a WHO Simplified Linearization
/// Output export.
///
/// Every variant carries the 1-based line number of the offending line
/// (counting the header as line 1), so callers can point a user back at the
/// exact row in the source file that failed to parse.
///
/// This type is `#[non_exhaustive]`: new variants may be added in minor
/// releases as more malformed shapes are discovered in the wild.
///
/// # Examples
///
/// ```
/// use who_fic_linearization::{LinearizationError, LinearizationReader};
///
/// // An unterminated quoted `Title` field on line 2.
/// let input = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC\nhttp://id.who.int/icd/entity/1\thttp://id.who.int/icd/release/11/beta/icf/1\t\t\t\"Unterminated title\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0\n";
/// let mut reader = LinearizationReader::from_str(input);
/// let result = reader.next().unwrap();
/// assert!(matches!(
///     result,
///     Err(LinearizationError::UnterminatedQuotedField { line: 2 })
/// ));
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinearizationError {
    /// Reading the underlying stream failed (e.g. an I/O error, or bytes
    /// that were not valid UTF-8).
    Io {
        /// The 1-based line the reader was attempting to read when the
        /// error occurred.
        line: usize,
        /// A human-readable description of the underlying error.
        message: String,
    },
    /// A quoted field (`"..."`) was opened but its closing quote was never
    /// found before the line ended.
    UnterminatedQuotedField {
        /// The 1-based line number.
        line: usize,
    },
    /// Extra characters appeared between a quoted field's closing quote and
    /// the next tab delimiter (or the end of the line).
    TrailingDataAfterQuotedField {
        /// The 1-based line number.
        line: usize,
    },
    /// A field expected to hold `True`/`False` held something else.
    InvalidBoolean {
        /// The 1-based line number.
        line: usize,
        /// The name of the column that failed to parse.
        field: &'static str,
        /// The raw text that could not be parsed as a boolean.
        found: String,
    },
    /// A field expected to hold an unsigned integer held something else.
    InvalidInteger {
        /// The 1-based line number.
        line: usize,
        /// The name of the column that failed to parse.
        field: &'static str,
        /// The raw text that could not be parsed as an integer.
        found: String,
    },
}

impl LinearizationError {
    /// The 1-based line number of the row that caused this error.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_linearization::LinearizationError;
    ///
    /// let error = LinearizationError::UnterminatedQuotedField { line: 42 };
    /// assert_eq!(error.line(), 42);
    /// ```
    pub fn line(&self) -> usize {
        match self {
            Self::Io { line, .. }
            | Self::UnterminatedQuotedField { line }
            | Self::TrailingDataAfterQuotedField { line }
            | Self::InvalidBoolean { line, .. }
            | Self::InvalidInteger { line, .. } => *line,
        }
    }
}

impl fmt::Display for LinearizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { line, message } => {
                write!(f, "line {line}: I/O error reading input: {message}")
            }
            Self::UnterminatedQuotedField { line } => {
                write!(f, "line {line}: unterminated quoted field")
            }
            Self::TrailingDataAfterQuotedField { line } => {
                write!(
                    f,
                    "line {line}: unexpected data after a quoted field's closing quote"
                )
            }
            Self::InvalidBoolean { line, field, found } => {
                write!(
                    f,
                    "line {line}: field {field:?} is not a valid boolean: {found:?}"
                )
            }
            Self::InvalidInteger { line, field, found } => {
                write!(
                    f,
                    "line {line}: field {field:?} is not a valid integer: {found:?}"
                )
            }
        }
    }
}

impl std::error::Error for LinearizationError {}
