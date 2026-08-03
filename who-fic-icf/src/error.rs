use std::fmt;

/// Error returned when parsing an ICF code (or qualifier) fails.
///
/// Shared shape across `who-fic-*` crates (see `specs/architecture.md`):
/// every variant reports either that the input was empty, had the wrong
/// length, contained a disallowed character at a known position, or had
/// the right characters but the wrong structure.
///
/// `#[non_exhaustive]` so new, more specific variants can be added without
/// a breaking change.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::{IcfCode, IcfParseError};
///
/// let err = IcfCode::from_str("x280").unwrap_err();
/// assert_eq!(
///     err,
///     IcfParseError::InvalidCharacter { position: 0, found: 'x' }
/// );
/// assert_eq!(err.to_string(), "invalid character 'x' at position 0");
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IcfParseError {
    /// The input was empty.
    Empty,
    /// The input had the wrong length.
    InvalidLength {
        /// A human-readable description of the expected length(s).
        expected: &'static str,
        /// The length actually found.
        found: usize,
    },
    /// The input contained a character not allowed at that position.
    InvalidCharacter {
        /// Zero-based character position of the offending character.
        position: usize,
        /// The offending character.
        found: char,
    },
    /// The input had the right characters but the wrong structure.
    InvalidStructure {
        /// A human-readable reason.
        reason: &'static str,
    },
}

impl fmt::Display for IcfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcfParseError::Empty => write!(f, "empty input"),
            IcfParseError::InvalidLength { expected, found } => {
                write!(f, "invalid length: expected {expected}, found {found}")
            }
            IcfParseError::InvalidCharacter { position, found } => {
                write!(f, "invalid character {found:?} at position {position}")
            }
            IcfParseError::InvalidStructure { reason } => {
                write!(f, "invalid structure: {reason}")
            }
        }
    }
}

impl std::error::Error for IcfParseError {}
