use crate::IcfParseError;
use std::fmt;
use std::str::FromStr;

/// One of the four ICF components.
///
/// These four are fixed by the ICF's design, so this enum is not
/// `#[non_exhaustive]`. Personal factors are part of the ICF model but are
/// *not classified* (no codes are assigned to them), so there is no
/// variant for them.
///
/// # Examples
///
/// ```
/// use who_fic_icf::Component;
///
/// assert_eq!(Component::BodyFunctions.letter(), 'b');
/// assert_eq!("B".parse::<Component>().unwrap(), Component::BodyFunctions);
/// assert_eq!(Component::BodyFunctions.to_string(), "b");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Component {
    /// Body functions (`b`): physiological functions of body systems.
    BodyFunctions,
    /// Body structures (`s`): anatomical parts of the body.
    BodyStructures,
    /// Activities and participation (`d`): execution of tasks and
    /// involvement in life situations.
    ActivitiesAndParticipation,
    /// Environmental factors (`e`): the physical, social, and attitudinal
    /// environment in which people live.
    EnvironmentalFactors,
}

impl Component {
    /// The canonical lowercase letter for this component.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::Component;
    ///
    /// assert_eq!(Component::BodyStructures.letter(), 's');
    /// assert_eq!(Component::EnvironmentalFactors.letter(), 'e');
    /// ```
    pub fn letter(&self) -> char {
        match self {
            Component::BodyFunctions => 'b',
            Component::BodyStructures => 's',
            Component::ActivitiesAndParticipation => 'd',
            Component::EnvironmentalFactors => 'e',
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.letter())
    }
}

impl TryFrom<char> for Component {
    type Error = IcfParseError;

    /// Parses a component letter, case-insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::Component;
    ///
    /// assert_eq!(Component::try_from('d').unwrap(), Component::ActivitiesAndParticipation);
    /// assert_eq!(Component::try_from('D').unwrap(), Component::ActivitiesAndParticipation);
    /// assert!(Component::try_from('x').is_err());
    /// ```
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'b' => Ok(Component::BodyFunctions),
            's' => Ok(Component::BodyStructures),
            'd' => Ok(Component::ActivitiesAndParticipation),
            'e' => Ok(Component::EnvironmentalFactors),
            _ => Err(IcfParseError::InvalidCharacter {
                position: 0,
                found: c,
            }),
        }
    }
}

impl FromStr for Component {
    type Err = IcfParseError;

    /// Parses a single-character component string, case-insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::Component;
    ///
    /// assert_eq!(Component::from_str("e").unwrap(), Component::EnvironmentalFactors);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(IcfParseError::Empty);
        }
        let mut chars = s.chars();
        let c = chars.next().unwrap();
        if chars.next().is_some() {
            return Err(IcfParseError::InvalidLength {
                expected: "1 character",
                found: s.chars().count(),
            });
        }
        Component::try_from(c)
    }
}

impl TryFrom<&str> for Component {
    type Error = IcfParseError;

    /// Parses a single-character component string, case-insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::Component;
    ///
    /// assert_eq!(Component::try_from("s").unwrap(), Component::BodyStructures);
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_letters_case_insensitively() {
        assert_eq!(Component::from_str("b").unwrap(), Component::BodyFunctions);
        assert_eq!(Component::from_str("B").unwrap(), Component::BodyFunctions);
        assert_eq!(Component::from_str("s").unwrap(), Component::BodyStructures);
        assert_eq!(Component::from_str("S").unwrap(), Component::BodyStructures);
        assert_eq!(
            Component::from_str("d").unwrap(),
            Component::ActivitiesAndParticipation
        );
        assert_eq!(
            Component::from_str("D").unwrap(),
            Component::ActivitiesAndParticipation
        );
        assert_eq!(
            Component::from_str("e").unwrap(),
            Component::EnvironmentalFactors
        );
        assert_eq!(
            Component::from_str("E").unwrap(),
            Component::EnvironmentalFactors
        );
    }

    #[test]
    fn rejects_unknown_letter() {
        assert_eq!(
            Component::from_str("x"),
            Err(IcfParseError::InvalidCharacter {
                position: 0,
                found: 'x'
            })
        );
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(Component::from_str(""), Err(IcfParseError::Empty));
    }

    #[test]
    fn rejects_multi_char() {
        assert!(matches!(
            Component::from_str("bs"),
            Err(IcfParseError::InvalidLength { .. })
        ));
    }

    #[test]
    fn display_is_lowercase() {
        assert_eq!(Component::BodyFunctions.to_string(), "b");
        assert_eq!(Component::BodyStructures.to_string(), "s");
        assert_eq!(Component::ActivitiesAndParticipation.to_string(), "d");
        assert_eq!(Component::EnvironmentalFactors.to_string(), "e");
    }

    #[test]
    fn ordering_matches_declaration_order() {
        assert!(Component::BodyFunctions < Component::BodyStructures);
        assert!(Component::BodyStructures < Component::ActivitiesAndParticipation);
        assert!(Component::ActivitiesAndParticipation < Component::EnvironmentalFactors);
    }
}
