//! World Health Organization (WHO) Family of International
//! Classifications (FIC).
//!
//! Umbrella crate re-exporting each classification behind a same-named
//! feature (all enabled by default):
//!
//! - [`icd`] — International Classification of Diseases (feature `icd`)
//! - [`icf`] — International Classification of Functioning, Disability and
//!   Health (feature `icf`)
//! - [`ichi`] — International Classification of Health Interventions
//!   (feature `ichi`)
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "icd")] {
//! use who_fic::icd::icd11::Icd11Code;
//! use std::str::FromStr;
//!
//! let code = Icd11Code::from_str("8B20").unwrap();
//! assert_eq!(code.as_str(), "8B20");
//! # }
//! ```

#![warn(missing_docs)]

#[cfg(feature = "icd")]
pub use who_fic_icd as icd;

#[cfg(feature = "icf")]
pub use who_fic_icf as icf;

#[cfg(feature = "ichi")]
pub use who_fic_ichi as ichi;

use std::fmt;
use std::str::FromStr;

/// A member of the WHO Family of International Classifications.
///
/// `#[non_exhaustive]` because WHO-FIC contains related and derived
/// classifications that may be added later.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Classification {
    /// International Classification of Diseases, 10th revision.
    Icd10,
    /// International Classification of Diseases, 11th revision.
    Icd11,
    /// International Classification of Functioning, Disability and Health.
    Icf,
    /// International Classification of Health Interventions.
    Ichi,
}

impl Classification {
    /// The conventional short name, e.g. `"ICD-11"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Classification::Icd10 => "ICD-10",
            Classification::Icd11 => "ICD-11",
            Classification::Icf => "ICF",
            Classification::Ichi => "ICHI",
        }
    }
}

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned when a string does not name a known [`Classification`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseClassificationError {
    found: String,
}

impl fmt::Display for ParseClassificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown WHO-FIC classification: {:?}", self.found)
    }
}

impl std::error::Error for ParseClassificationError {}

impl FromStr for Classification {
    type Err = ParseClassificationError;

    /// Accepts the short name case-insensitively, with or without a
    /// hyphen (e.g. `"icd11"`, `"ICD-11"`, `"Icd-11"`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized: String = s.chars().filter(|c| *c != '-').collect();
        match normalized.to_ascii_uppercase().as_str() {
            "ICD10" => Ok(Classification::Icd10),
            "ICD11" => Ok(Classification::Icd11),
            "ICF" => Ok(Classification::Icf),
            "ICHI" => Ok(Classification::Ichi),
            _ => Err(ParseClassificationError {
                found: s.to_string(),
            }),
        }
    }
}

/// Shared parse-error shape for WHO-FIC classification codes.
///
/// Subcrates define structurally identical error types of their own so
/// they remain independent of this umbrella crate; conversions from each
/// are provided below, gated on the matching feature.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FicError {
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

impl fmt::Display for FicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FicError::Empty => write!(f, "empty input"),
            FicError::InvalidLength { expected, found } => {
                write!(f, "invalid length: expected {expected}, found {found}")
            }
            FicError::InvalidCharacter { position, found } => {
                write!(f, "invalid character {found:?} at position {position}")
            }
            FicError::InvalidStructure { reason } => {
                write!(f, "invalid structure: {reason}")
            }
        }
    }
}

impl std::error::Error for FicError {}

#[cfg(feature = "icd")]
impl From<who_fic_icd::icd10::Icd10ParseError> for FicError {
    fn from(error: who_fic_icd::icd10::Icd10ParseError) -> Self {
        use who_fic_icd::icd10::Icd10ParseError as E;
        match error {
            E::Empty => FicError::Empty,
            E::InvalidLength { found } => FicError::InvalidLength {
                expected: "ICD-10 code, e.g. A00 or A00.0",
                found,
            },
            E::InvalidCharacter { position, found } => {
                FicError::InvalidCharacter { position, found }
            }
            E::InvalidStructure { reason } => FicError::InvalidStructure { reason },
            _ => FicError::InvalidStructure {
                reason: "unrecognized ICD-10 parse error variant",
            },
        }
    }
}

#[cfg(feature = "icd")]
impl From<who_fic_icd::icd11::Icd11ParseError> for FicError {
    fn from(error: who_fic_icd::icd11::Icd11ParseError) -> Self {
        use who_fic_icd::icd11::Icd11ParseError as E;
        match error {
            E::Empty => FicError::Empty,
            E::InvalidLength { found } => FicError::InvalidLength {
                expected: "ICD-11 code, e.g. 8B20 or CA40.0",
                found,
            },
            E::InvalidCharacter { position, found } => {
                FicError::InvalidCharacter { position, found }
            }
            E::InvalidStructure { reason } => FicError::InvalidStructure { reason },
            _ => FicError::InvalidStructure {
                reason: "unrecognized ICD-11 parse error variant",
            },
        }
    }
}

#[cfg(feature = "icf")]
impl From<who_fic_icf::IcfParseError> for FicError {
    fn from(error: who_fic_icf::IcfParseError) -> Self {
        use who_fic_icf::IcfParseError as E;
        match error {
            E::Empty => FicError::Empty,
            E::InvalidLength { expected, found } => FicError::InvalidLength { expected, found },
            E::InvalidCharacter { position, found } => {
                FicError::InvalidCharacter { position, found }
            }
            E::InvalidStructure { reason } => FicError::InvalidStructure { reason },
            _ => FicError::InvalidStructure {
                reason: "unrecognized ICF parse error variant",
            },
        }
    }
}

// `who_fic_ichi::IchiParseError` carries an extra `axis: Option<Axis>` on
// every variant, pinpointing which dot-segment of a `TARGET.ACTION.MEANS`
// code failed. `FicError` has no equivalent field, so that context is
// dropped here; callers who need it should match on
// `who_fic_ichi::IchiParseError` directly instead of going through
// `FicError`.
#[cfg(feature = "ichi")]
impl From<who_fic_ichi::IchiParseError> for FicError {
    fn from(error: who_fic_ichi::IchiParseError) -> Self {
        use who_fic_ichi::IchiParseError as E;
        match error {
            E::Empty { .. } => FicError::Empty,
            E::InvalidLength {
                expected, found, ..
            } => FicError::InvalidLength { expected, found },
            E::InvalidCharacter {
                position, found, ..
            } => FicError::InvalidCharacter { position, found },
            E::InvalidStructure { reason, .. } => FicError::InvalidStructure { reason },
            _ => FicError::InvalidStructure {
                reason: "unrecognized ICHI parse error variant",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "icd")]
    fn icd_reexport_resolves() {
        use std::str::FromStr;
        assert!(crate::icd::icd11::Icd11Code::from_str("8B20").is_ok());
    }

    #[test]
    #[cfg(feature = "icf")]
    fn icf_reexport_resolves() {
        use std::str::FromStr;
        assert!(crate::icf::IcfCode::from_str("b280").is_ok());
    }

    #[test]
    #[cfg(feature = "ichi")]
    fn ichi_reexport_resolves() {
        use std::str::FromStr;
        assert!(crate::ichi::IchiCode::from_str("KAB.DB.AD").is_ok());
    }

    #[test]
    #[cfg(feature = "icd")]
    fn icd10_error_conversion_preserves_variant_meaning() {
        use std::str::FromStr;
        let error = who_fic_icd::icd10::Icd10Code::from_str("").unwrap_err();
        assert_eq!(FicError::from(error), FicError::Empty);

        let error = who_fic_icd::icd10::Icd10Code::from_str("a0!").unwrap_err();
        assert!(matches!(
            FicError::from(error),
            FicError::InvalidCharacter { .. }
        ));
    }

    #[test]
    #[cfg(feature = "icd")]
    fn icd11_error_conversion_preserves_variant_meaning() {
        use std::str::FromStr;
        let error = who_fic_icd::icd11::Icd11Code::from_str("").unwrap_err();
        assert_eq!(FicError::from(error), FicError::Empty);
    }

    #[test]
    #[cfg(feature = "icf")]
    fn icf_error_conversion_preserves_variant_meaning() {
        use std::str::FromStr;
        let error = who_fic_icf::IcfCode::from_str("").unwrap_err();
        assert_eq!(FicError::from(error), FicError::Empty);
    }

    #[test]
    #[cfg(feature = "ichi")]
    fn ichi_error_conversion_preserves_variant_meaning() {
        use std::str::FromStr;
        let error = who_fic_ichi::IchiCode::from_str("").unwrap_err();
        assert_eq!(FicError::from(error), FicError::Empty);
    }

    #[test]
    fn classification_round_trip() {
        for c in [
            Classification::Icd10,
            Classification::Icd11,
            Classification::Icf,
            Classification::Ichi,
        ] {
            assert_eq!(Classification::from_str(c.as_str()).unwrap(), c);
            assert_eq!(
                Classification::from_str(&c.as_str().to_lowercase()).unwrap(),
                c
            );
        }
    }

    #[test]
    fn classification_rejects_unknown() {
        assert!(Classification::from_str("nope").is_err());
    }
}
