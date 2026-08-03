use crate::IcfParseError;
use std::fmt;
use std::str::FromStr;

/// A single ICF qualifier digit: the WHO "generic scale" used to record
/// the extent of a problem (or, on the environmental-factors component,
/// the extent of a barrier or facilitator).
///
/// | Digit | Meaning |
/// |---|---|
/// | 0 | `No` — NO problem (none, absent, negligible) |
/// | 1 | `Mild` — MILD problem |
/// | 2 | `Moderate` — MODERATE problem |
/// | 3 | `Severe` — SEVERE problem |
/// | 4 | `Complete` — COMPLETE problem |
/// | 8 | `NotSpecified` — not specified |
/// | 9 | `NotApplicable` — not applicable |
///
/// Digits 5–7 are not part of the scale and are rejected.
///
/// Variants are declared in ascending digit order, so the derived
/// [`Ord`] matches numeric qualifier order (`No < Mild < ... < NotSpecified
/// < NotApplicable`).
///
/// # Examples
///
/// ```
/// use who_fic_icf::Qualifier;
///
/// let q: Qualifier = "2".parse().unwrap();
/// assert_eq!(q, Qualifier::Moderate);
/// assert_eq!(q.as_digit(), 2);
/// assert_eq!(q.to_string(), "2");
/// assert!("5".parse::<Qualifier>().is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Qualifier {
    /// Digit `0`: no problem.
    No,
    /// Digit `1`: mild problem.
    Mild,
    /// Digit `2`: moderate problem.
    Moderate,
    /// Digit `3`: severe problem.
    Severe,
    /// Digit `4`: complete problem.
    Complete,
    /// Digit `8`: not specified.
    NotSpecified,
    /// Digit `9`: not applicable.
    NotApplicable,
}

impl Qualifier {
    /// The digit this qualifier is written as in a code.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::Qualifier;
    ///
    /// assert_eq!(Qualifier::Complete.as_digit(), 4);
    /// assert_eq!(Qualifier::NotApplicable.as_digit(), 9);
    /// ```
    pub fn as_digit(&self) -> u8 {
        match self {
            Qualifier::No => 0,
            Qualifier::Mild => 1,
            Qualifier::Moderate => 2,
            Qualifier::Severe => 3,
            Qualifier::Complete => 4,
            Qualifier::NotSpecified => 8,
            Qualifier::NotApplicable => 9,
        }
    }
}

impl fmt::Display for Qualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_digit())
    }
}

impl TryFrom<char> for Qualifier {
    type Error = IcfParseError;

    /// Parses a single qualifier digit.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::Qualifier;
    ///
    /// assert_eq!(Qualifier::try_from('0').unwrap(), Qualifier::No);
    /// assert!(Qualifier::try_from('5').is_err());
    /// ```
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Qualifier::No),
            '1' => Ok(Qualifier::Mild),
            '2' => Ok(Qualifier::Moderate),
            '3' => Ok(Qualifier::Severe),
            '4' => Ok(Qualifier::Complete),
            '8' => Ok(Qualifier::NotSpecified),
            '9' => Ok(Qualifier::NotApplicable),
            _ => Err(IcfParseError::InvalidCharacter {
                position: 0,
                found: c,
            }),
        }
    }
}

impl FromStr for Qualifier {
    type Err = IcfParseError;

    /// Parses a single-digit qualifier string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::Qualifier;
    ///
    /// assert_eq!(Qualifier::from_str("9").unwrap(), Qualifier::NotApplicable);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(IcfParseError::Empty);
        }
        let mut chars = s.chars();
        let c = chars.next().unwrap();
        if chars.next().is_some() {
            return Err(IcfParseError::InvalidLength {
                expected: "1 digit",
                found: s.chars().count(),
            });
        }
        Qualifier::try_from(c)
    }
}

impl TryFrom<&str> for Qualifier {
    type Error = IcfParseError;

    /// Parses a single-digit qualifier string. See `Qualifier::from_str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::Qualifier;
    ///
    /// assert_eq!(Qualifier::try_from("1").unwrap(), Qualifier::Mild);
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_all_valid_digits() {
        let expected = [
            ('0', Qualifier::No),
            ('1', Qualifier::Mild),
            ('2', Qualifier::Moderate),
            ('3', Qualifier::Severe),
            ('4', Qualifier::Complete),
            ('8', Qualifier::NotSpecified),
            ('9', Qualifier::NotApplicable),
        ];
        for (digit, q) in expected {
            assert_eq!(Qualifier::try_from(digit).unwrap(), q);
            assert_eq!(q.as_digit(), digit.to_digit(10).unwrap() as u8);
            assert_eq!(q.to_string(), digit.to_string());
        }
    }

    #[test]
    fn rejects_invalid_digits() {
        for digit in ['5', '6', '7'] {
            assert_eq!(
                Qualifier::try_from(digit),
                Err(IcfParseError::InvalidCharacter {
                    position: 0,
                    found: digit
                })
            );
        }
    }

    #[test]
    fn rejects_non_digit() {
        assert!(Qualifier::try_from('a').is_err());
    }

    #[test]
    fn rejects_empty_and_multi_char() {
        assert_eq!(Qualifier::from_str(""), Err(IcfParseError::Empty));
        assert!(matches!(
            Qualifier::from_str("12"),
            Err(IcfParseError::InvalidLength { .. })
        ));
    }

    #[test]
    fn ordering_matches_digit_order() {
        assert!(Qualifier::No < Qualifier::Mild);
        assert!(Qualifier::Complete < Qualifier::NotSpecified);
        assert!(Qualifier::NotSpecified < Qualifier::NotApplicable);
    }
}
