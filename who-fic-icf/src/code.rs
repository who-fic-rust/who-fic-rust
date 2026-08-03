use crate::{Component, IcfParseError, Level};
use std::fmt;
use std::str::FromStr;

/// An unqualified ICF hierarchy code, e.g. `b280`.
///
/// Grammar (see `specs/who-fic-icf.md`):
///
/// ```text
/// code      = component digits
/// component = "b" / "s" / "d" / "e"
/// digits    = 1 digit    ; chapter      (level 1), e.g. b2
///           / 3 digits   ; second level,           e.g. b280
///           / 4 digits   ; third level,            e.g. b2801
///           / 5 digits   ; fourth level,           e.g. b28010
/// ```
///
/// Two digits is invalid: there is no 2-digit level.
///
/// This is the *unqualified* form that appears in the tabulation itself.
/// For a code carrying an assessment qualifier (e.g. `b280.2`), see
/// [`QualifiedIcfCode`](crate::QualifiedIcfCode).
///
/// The canonical form is the lowercase component letter followed by the
/// digits, with no separator (`b280`, not `B280`). Parsing accepts the
/// component letter case-insensitively but always canonicalizes to
/// lowercase.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::{Component, IcfCode};
///
/// let code = IcfCode::from_str("B280").unwrap();
/// assert_eq!(code.as_str(), "b280");
/// assert_eq!(code.component(), Component::BodyFunctions);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IcfCode {
    code: String,
}

impl IcfCode {
    /// The canonical string form of this code.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::IcfCode;
    ///
    /// assert_eq!(IcfCode::from_str("s730").unwrap().as_str(), "s730");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.code
    }

    /// The component this code belongs to.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::{Component, IcfCode};
    ///
    /// assert_eq!(
    ///     IcfCode::from_str("d450").unwrap().component(),
    ///     Component::ActivitiesAndParticipation
    /// );
    /// ```
    pub fn component(&self) -> Component {
        let first = self.code.chars().next().expect("IcfCode is never empty");
        Component::try_from(first).expect("IcfCode always starts with a valid component letter")
    }

    /// The digits after the component letter.
    fn digits(&self) -> &str {
        &self.code[1..]
    }

    /// The hierarchy depth of this code.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::{IcfCode, Level};
    ///
    /// assert_eq!(IcfCode::from_str("b2").unwrap().level(), Level::Chapter);
    /// assert_eq!(IcfCode::from_str("b280").unwrap().level(), Level::SecondLevel);
    /// assert_eq!(IcfCode::from_str("b2801").unwrap().level(), Level::ThirdLevel);
    /// assert_eq!(IcfCode::from_str("b28010").unwrap().level(), Level::FourthLevel);
    /// ```
    pub fn level(&self) -> Level {
        match self.digits().len() {
            1 => Level::Chapter,
            3 => Level::SecondLevel,
            4 => Level::ThirdLevel,
            5 => Level::FourthLevel,
            n => unreachable!("IcfCode digit count is always 1, 3, 4, or 5, got {n}"),
        }
    }

    /// The immediate parent of this code, truncating one hierarchy level,
    /// or `None` if this code is already a chapter.
    ///
    /// `b28010` → `b2801` → `b280` → `b2` → `None`. Note that the step from
    /// second level to chapter drops two digits (there is no 2-digit
    /// level), while every other step drops one.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::IcfCode;
    ///
    /// let code = IcfCode::from_str("b28010").unwrap();
    /// assert_eq!(code.parent().unwrap().as_str(), "b2801");
    ///
    /// let chapter = IcfCode::from_str("b2").unwrap();
    /// assert_eq!(chapter.parent(), None);
    /// ```
    pub fn parent(&self) -> Option<IcfCode> {
        let digits = self.digits();
        let truncated_len = match digits.len() {
            5 => 4,
            4 => 3,
            3 => 1,
            1 => return None,
            n => unreachable!("IcfCode digit count is always 1, 3, 4, or 5, got {n}"),
        };
        let mut code = String::with_capacity(1 + truncated_len);
        code.push(self.component().letter());
        code.push_str(&digits[..truncated_len]);
        Some(IcfCode { code })
    }

    /// The chapter (level 1) ancestor of this code, truncating straight to
    /// the first digit.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::IcfCode;
    ///
    /// let code = IcfCode::from_str("b28010").unwrap();
    /// assert_eq!(code.chapter().as_str(), "b2");
    /// ```
    pub fn chapter(&self) -> IcfCode {
        let mut code = String::with_capacity(2);
        code.push(self.component().letter());
        code.push(self.digits().chars().next().expect("digits is never empty"));
        IcfCode { code }
    }

    /// Whether `self` is a (possibly indirect) ancestor of `other` in the
    /// hierarchy, i.e. `other` can be reached from `self` by zero or more
    /// [`parent`](IcfCode::parent) steps in reverse.
    ///
    /// A code is not considered an ancestor of itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::IcfCode;
    ///
    /// let chapter = IcfCode::from_str("b2").unwrap();
    /// let leaf = IcfCode::from_str("b28010").unwrap();
    /// assert!(chapter.is_ancestor_of(&leaf));
    /// assert!(!leaf.is_ancestor_of(&chapter));
    /// assert!(!chapter.is_ancestor_of(&chapter));
    /// ```
    pub fn is_ancestor_of(&self, other: &IcfCode) -> bool {
        let mut current = other.clone();
        while let Some(parent) = current.parent() {
            if parent == *self {
                return true;
            }
            current = parent;
        }
        false
    }

    /// Whether `self` is a (possibly indirect) descendant of `other`.
    /// The inverse of [`is_ancestor_of`](IcfCode::is_ancestor_of).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::IcfCode;
    ///
    /// let chapter = IcfCode::from_str("b2").unwrap();
    /// let leaf = IcfCode::from_str("b28010").unwrap();
    /// assert!(leaf.is_descendant_of(&chapter));
    /// assert!(!chapter.is_descendant_of(&leaf));
    /// ```
    pub fn is_descendant_of(&self, other: &IcfCode) -> bool {
        other.is_ancestor_of(self)
    }
}

impl fmt::Display for IcfCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code)
    }
}

impl FromStr for IcfCode {
    type Err = IcfParseError;

    /// Parses an unqualified ICF code.
    ///
    /// The component letter is accepted case-insensitively; the digit
    /// count must be 1, 3, 4, or 5.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::{IcfCode, IcfParseError};
    ///
    /// assert!(IcfCode::from_str("b280").is_ok());
    /// assert_eq!(
    ///     IcfCode::from_str("b28").unwrap_err(),
    ///     IcfParseError::InvalidLength {
    ///         expected: "1, 3, 4, or 5 digits after the component letter",
    ///         found: 2,
    ///     }
    /// );
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(IcfParseError::Empty);
        }
        let first = s.chars().next().unwrap();
        let component =
            Component::try_from(first).map_err(|_| IcfParseError::InvalidCharacter {
                position: 0,
                found: first,
            })?;
        // The component letter is always a single-byte ASCII character.
        let rest = &s[1..];
        if rest.is_empty() {
            return Err(IcfParseError::InvalidLength {
                expected: "1, 3, 4, or 5 digits after the component letter",
                found: 0,
            });
        }
        for (i, c) in rest.chars().enumerate() {
            if !c.is_ascii_digit() {
                return Err(IcfParseError::InvalidCharacter {
                    position: i + 1,
                    found: c,
                });
            }
        }
        let digit_count = rest.chars().count();
        if !matches!(digit_count, 1 | 3 | 4 | 5) {
            return Err(IcfParseError::InvalidLength {
                expected: "1, 3, 4, or 5 digits after the component letter",
                found: digit_count,
            });
        }
        let mut code = String::with_capacity(1 + digit_count);
        code.push(component.letter());
        code.push_str(rest);
        Ok(IcfCode { code })
    }
}

impl TryFrom<&str> for IcfCode {
    type Error = IcfParseError;

    /// Parses an unqualified ICF code. See `IcfCode::from_str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::IcfCode;
    ///
    /// assert!(IcfCode::try_from("s7301").is_ok());
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Serializes as the canonical string form (via [`Display`](fmt::Display)),
/// not as a struct.
#[cfg(feature = "serde")]
impl serde::Serialize for IcfCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Deserializes from the canonical string form (via [`FromStr`]).
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IcfCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCEPT: &[&str] = &["b2", "b280", "b2801", "b28010", "s730", "d450", "e150"];

    #[test]
    fn accepts_valid_codes() {
        for s in ACCEPT {
            assert!(IcfCode::from_str(s).is_ok(), "expected {s} to parse");
        }
    }

    #[test]
    fn canonicalizes_case() {
        assert_eq!(IcfCode::from_str("B280").unwrap().as_str(), "b280");
        assert_eq!(IcfCode::from_str("S730").unwrap().as_str(), "s730");
    }

    #[test]
    fn rejects_two_digits() {
        assert_eq!(
            IcfCode::from_str("b28"),
            Err(IcfParseError::InvalidLength {
                expected: "1, 3, 4, or 5 digits after the component letter",
                found: 2,
            })
        );
    }

    #[test]
    fn rejects_bad_component_letter() {
        assert_eq!(
            IcfCode::from_str("x280"),
            Err(IcfParseError::InvalidCharacter {
                position: 0,
                found: 'x'
            })
        );
    }

    #[test]
    fn rejects_non_digit_in_digits() {
        assert_eq!(
            IcfCode::from_str("b2a0"),
            Err(IcfParseError::InvalidCharacter {
                position: 2,
                found: 'a'
            })
        );
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(IcfCode::from_str(""), Err(IcfParseError::Empty));
    }

    #[test]
    fn rejects_component_only() {
        assert_eq!(
            IcfCode::from_str("b"),
            Err(IcfParseError::InvalidLength {
                expected: "1, 3, 4, or 5 digits after the component letter",
                found: 0,
            })
        );
    }

    #[test]
    fn rejects_six_digits() {
        assert!(matches!(
            IcfCode::from_str("b280101"),
            Err(IcfParseError::InvalidLength { .. })
        ));
    }

    #[test]
    fn parent_chain_all_levels() {
        let leaf = IcfCode::from_str("b28010").unwrap();
        let third = leaf.parent().unwrap();
        assert_eq!(third.as_str(), "b2801");
        let second = third.parent().unwrap();
        assert_eq!(second.as_str(), "b280");
        let chapter = second.parent().unwrap();
        assert_eq!(chapter.as_str(), "b2");
        assert_eq!(chapter.parent(), None);
    }

    #[test]
    fn chapter_truncates_straight_to_level_one() {
        let leaf = IcfCode::from_str("b28010").unwrap();
        assert_eq!(leaf.chapter().as_str(), "b2");
    }

    #[test]
    fn ancestor_descendant_relationships() {
        let chapter = IcfCode::from_str("b2").unwrap();
        let second = IcfCode::from_str("b280").unwrap();
        let third = IcfCode::from_str("b2801").unwrap();
        let fourth = IcfCode::from_str("b28010").unwrap();

        assert!(chapter.is_ancestor_of(&second));
        assert!(chapter.is_ancestor_of(&third));
        assert!(chapter.is_ancestor_of(&fourth));
        assert!(second.is_ancestor_of(&third));
        assert!(second.is_ancestor_of(&fourth));
        assert!(third.is_ancestor_of(&fourth));

        assert!(fourth.is_descendant_of(&third));
        assert!(fourth.is_descendant_of(&second));
        assert!(fourth.is_descendant_of(&chapter));

        assert!(!fourth.is_ancestor_of(&chapter));
        assert!(!chapter.is_ancestor_of(&chapter));

        // Different components never relate, even with matching digits.
        let other_component = IcfCode::from_str("s280").unwrap();
        assert!(!chapter.is_ancestor_of(&other_component));
    }

    #[test]
    fn levels_are_correct() {
        assert_eq!(IcfCode::from_str("b2").unwrap().level(), Level::Chapter);
        assert_eq!(
            IcfCode::from_str("b280").unwrap().level(),
            Level::SecondLevel
        );
        assert_eq!(
            IcfCode::from_str("b2801").unwrap().level(),
            Level::ThirdLevel
        );
        assert_eq!(
            IcfCode::from_str("b28010").unwrap().level(),
            Level::FourthLevel
        );
    }

    #[test]
    fn round_trip() {
        for s in ACCEPT {
            let code = IcfCode::from_str(s).unwrap();
            assert_eq!(IcfCode::from_str(code.as_str()).unwrap(), code);
        }
    }
}
