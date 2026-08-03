//! [`IchiCode`]: the composed `TARGET.ACTION.MEANS` intervention code.

use std::fmt;
use std::str::FromStr;

use crate::axis::{Action, Means, Target};
use crate::error::IchiParseError;
use crate::section::Section;

/// A composed ICHI intervention code: `TARGET.ACTION.MEANS`, e.g.
/// `KAB.DB.AD`.
///
/// Built from the three ICHI axes — [`Target`] (3 characters), a literal
/// `.`, [`Action`] (2 characters), a literal `.`, and [`Means`] (2
/// characters) — for a canonical uppercase form of 9 characters total.
///
/// # Examples
///
/// ```
/// use who_fic_ichi::IchiCode;
///
/// let code: IchiCode = "kab.db.ad".parse().unwrap();
/// assert_eq!(code.to_string(), "KAB.DB.AD");
/// assert_eq!(code.target().as_str(), "KAB");
/// assert_eq!(code.action().as_str(), "DB");
/// assert_eq!(code.means().as_str(), "AD");
/// ```
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IchiCode {
    target: Target,
    action: Action,
    means: Means,
    canonical: String,
}

impl IchiCode {
    /// Composes an [`IchiCode`] from its three already-validated axis
    /// values.
    ///
    /// Infallible: once `target`, `action`, and `means` each exist as valid
    /// axis values, their composition into a dotted code has no further
    /// constraints at the syntax level.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::{Action, IchiCode, Means, Target};
    ///
    /// let target: Target = "KAB".parse().unwrap();
    /// let action: Action = "DB".parse().unwrap();
    /// let means: Means = "AD".parse().unwrap();
    /// let code = IchiCode::from_parts(target, action, means);
    /// assert_eq!(code.to_string(), "KAB.DB.AD");
    /// ```
    pub fn from_parts(target: Target, action: Action, means: Means) -> Self {
        let canonical = format!("{target}.{action}.{means}");
        Self {
            target,
            action,
            means,
            canonical,
        }
    }

    /// Returns the [`Target`] axis of this code.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::IchiCode;
    ///
    /// let code: IchiCode = "KAB.DB.AD".parse().unwrap();
    /// assert_eq!(code.target().as_str(), "KAB");
    /// ```
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// Returns the [`Action`] axis of this code.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::IchiCode;
    ///
    /// let code: IchiCode = "KAB.DB.AD".parse().unwrap();
    /// assert_eq!(code.action().as_str(), "DB");
    /// ```
    pub fn action(&self) -> &Action {
        &self.action
    }

    /// Returns the [`Means`] axis of this code.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::IchiCode;
    ///
    /// let code: IchiCode = "KAB.DB.AD".parse().unwrap();
    /// assert_eq!(code.means().as_str(), "AD");
    /// ```
    pub fn means(&self) -> &Means {
        &self.means
    }

    /// Returns the canonical uppercase dotted string form of this code,
    /// e.g. `"KAB.DB.AD"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::IchiCode;
    ///
    /// let code: IchiCode = "kab.db.ad".parse().unwrap();
    /// assert_eq!(code.as_str(), "KAB.DB.AD");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// Returns the [`Section`] this code's target belongs to, if known.
    ///
    /// Delegates to [`Target::section`]; see that method and the [`Section`]
    /// documentation for why this currently always returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::IchiCode;
    ///
    /// let code: IchiCode = "KAB.DB.AD".parse().unwrap();
    /// assert_eq!(code.section(), None);
    /// ```
    pub fn section(&self) -> Option<Section> {
        self.target.section()
    }
}

impl fmt::Debug for IchiCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IchiCode").field(&self.canonical).finish()
    }
}

impl fmt::Display for IchiCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical)
    }
}

impl FromStr for IchiCode {
    type Err = IchiParseError;

    /// Parses the dotted `TARGET.ACTION.MEANS` form. On failure, the error
    /// identifies which axis (if any single axis is to blame) caused the
    /// failure; see [`IchiParseError`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(IchiParseError::Empty { axis: None });
        }
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(IchiParseError::InvalidStructure {
                reason: "expected exactly three dot-separated segments: TARGET.ACTION.MEANS",
                axis: None,
            });
        }
        let target: Target = parts[0].parse()?;
        let action: Action = parts[1].parse()?;
        let means: Means = parts[2].parse()?;
        Ok(IchiCode::from_parts(target, action, means))
    }
}

impl TryFrom<&str> for IchiCode {
    type Error = IchiParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for IchiCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IchiCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Axis;

    /// Illustrative codes matching the ICHI grammar, spanning what this
    /// crate believes are the three sections' target ranges. These are
    /// synthetic examples chosen to exercise the parser's grammar rules;
    /// they are not asserted to be codes WHO has actually assigned, except
    /// `KAB.DB.AD` which is the worked example from the spec /
    /// `specs/who-fic-ichi.md`.
    const ACCEPT_LIST: &[&str] = &[
        "KAB.DB.AD", // spec's worked example
        "AAA.FA.AE", // synthetic: nervous-system-shaped target
        "VBA.PQ.ZZ", // synthetic: activities/participation-shaped target
        "XZZ.AA.00", // synthetic: environment-shaped target
        "000.00.00", // synthetic: all-numeric segments are still valid alnum
    ];

    #[test]
    fn accepts_illustrative_codes() {
        for s in ACCEPT_LIST {
            let code: IchiCode = s
                .parse()
                .unwrap_or_else(|e| panic!("{s} should parse: {e}"));
            assert_eq!(code.to_string(), *s);
            assert_eq!(code.as_str(), *s);
        }
    }

    #[test]
    fn rejects_wrong_segment_lengths() {
        assert_eq!(
            "KA.DB.AD".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "3",
                found: 2,
                axis: Some(Axis::Target)
            }
        );
        assert_eq!(
            "KAB.D.AD".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "2",
                found: 1,
                axis: Some(Axis::Action)
            }
        );
        assert_eq!(
            "KAB.DB.A".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "2",
                found: 1,
                axis: Some(Axis::Means)
            }
        );
    }

    #[test]
    fn rejects_missing_or_wrong_separators() {
        assert_eq!(
            "KAB-DB-AD".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidStructure {
                reason: "expected exactly three dot-separated segments: TARGET.ACTION.MEANS",
                axis: None,
            }
        );
        assert_eq!(
            "KABDBAD".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidStructure {
                reason: "expected exactly three dot-separated segments: TARGET.ACTION.MEANS",
                axis: None,
            }
        );
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(
            "K@B.DB.AD".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidCharacter {
                position: 1,
                found: '@',
                axis: Some(Axis::Target)
            }
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(
            "".parse::<IchiCode>().unwrap_err(),
            IchiParseError::Empty { axis: None }
        );
    }

    #[test]
    fn rejects_trailing_garbage() {
        // Trailing garbage on the last segment surfaces as an invalid
        // length on the Means axis, since there's no fourth separator to
        // delimit it as its own segment.
        assert_eq!(
            "KAB.DB.ADXX".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "2",
                found: 4,
                axis: Some(Axis::Means)
            }
        );
        // A fourth dot-separated segment is a structural error.
        assert_eq!(
            "KAB.DB.AD.XX".parse::<IchiCode>().unwrap_err(),
            IchiParseError::InvalidStructure {
                reason: "expected exactly three dot-separated segments: TARGET.ACTION.MEANS",
                axis: None,
            }
        );
    }

    #[test]
    fn canonicalizes_lowercase_input() {
        let code: IchiCode = "kab.db.ad".parse().unwrap();
        assert_eq!(code.to_string(), "KAB.DB.AD");
        assert_eq!(code.as_str(), "KAB.DB.AD");
    }

    #[test]
    fn from_parts_composed_with_accessors_round_trips() {
        let target: Target = "KAB".parse().unwrap();
        let action: Action = "DB".parse().unwrap();
        let means: Means = "AD".parse().unwrap();
        let code = IchiCode::from_parts(target.clone(), action.clone(), means.clone());

        assert_eq!(code.target(), &target);
        assert_eq!(code.action(), &action);
        assert_eq!(code.means(), &means);
        assert_eq!(code.to_string(), "KAB.DB.AD");

        let reparsed: IchiCode = code.to_string().parse().unwrap();
        assert_eq!(code, reparsed);
    }

    #[test]
    fn try_from_matches_from_str() {
        assert_eq!(
            IchiCode::try_from("KAB.DB.AD").unwrap(),
            "KAB.DB.AD".parse::<IchiCode>().unwrap()
        );
    }

    #[test]
    fn section_is_none_pending_verified_range_table() {
        let code: IchiCode = "KAB.DB.AD".parse().unwrap();
        assert_eq!(code.section(), None);
    }

    #[test]
    fn ordering_matches_canonical_string_ordering() {
        let a: IchiCode = "AAA.AA.AA".parse().unwrap();
        let b: IchiCode = "AAB.AA.AA".parse().unwrap();
        assert!(a < b);
        assert!(a.to_string() < b.to_string());
    }

    #[test]
    fn debug_contains_canonical_form() {
        let code: IchiCode = "KAB.DB.AD".parse().unwrap();
        assert!(format!("{code:?}").contains("KAB.DB.AD"));
    }
}
