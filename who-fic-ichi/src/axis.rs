//! The three ICHI axis types: [`Target`], [`Action`], [`Means`].

use std::fmt;
use std::str::FromStr;

use crate::error::{Axis, IchiParseError};
use crate::section::Section;

/// Validates and canonicalizes an axis segment: exactly `len`
/// uppercase-alphanumeric characters, case-insensitive on input.
fn parse_axis(s: &str, len: usize, axis: Axis) -> Result<String, IchiParseError> {
    if s.is_empty() {
        return Err(IchiParseError::Empty { axis: Some(axis) });
    }
    let found = s.chars().count();
    if found != len {
        return Err(IchiParseError::InvalidLength {
            expected: match len {
                2 => "2",
                3 => "3",
                _ => "?",
            },
            found,
            axis: Some(axis),
        });
    }
    let mut out = String::with_capacity(len);
    for (position, c) in s.chars().enumerate() {
        if !c.is_ascii_alphanumeric() {
            return Err(IchiParseError::InvalidCharacter {
                position,
                found: c,
                axis: Some(axis),
            });
        }
        out.push(c.to_ascii_uppercase());
    }
    Ok(out)
}

/// The **Target** axis: the entity on which an [`Action`](crate::Action) is
/// carried out (anatomy, body function, activity domain, environment, …).
///
/// A validated newtype over exactly 3 uppercase-alphanumeric characters
/// (`[A-Z0-9]{3}`), e.g. `"KAB"`. Parsing is case-insensitive; the canonical
/// form is uppercase.
///
/// Target codes are published by WHO as a standalone value set (not merely
/// an internal detail of [`IchiCode`](crate::IchiCode)), so this type is a
/// public, independently useful code type in its own right.
///
/// # Examples
///
/// ```
/// use who_fic_ichi::Target;
///
/// let target: Target = "kab".parse().unwrap();
/// assert_eq!(target.as_str(), "KAB");
/// assert_eq!(target.to_string(), "KAB");
/// ```
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Target(String);

/// The **Action** axis: the deed done by the actor to the
/// [`Target`](crate::Target) (e.g. excision, education, assessment).
///
/// A validated newtype over exactly 2 uppercase-alphanumeric characters
/// (`[A-Z0-9]{2}`), e.g. `"DB"`. Parsing is case-insensitive; the canonical
/// form is uppercase.
///
/// # Examples
///
/// ```
/// use who_fic_ichi::Action;
///
/// let action: Action = "db".parse().unwrap();
/// assert_eq!(action.as_str(), "DB");
/// assert_eq!(action.to_string(), "DB");
/// ```
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Action(String);

/// The **Means** axis: the processes and methods by which the
/// [`Action`](crate::Action) is carried out (e.g. open approach,
/// endoscopic, instrument).
///
/// A validated newtype over exactly 2 uppercase-alphanumeric characters
/// (`[A-Z0-9]{2}`), e.g. `"AD"`. Parsing is case-insensitive; the canonical
/// form is uppercase.
///
/// # Examples
///
/// ```
/// use who_fic_ichi::Means;
///
/// let means: Means = "ad".parse().unwrap();
/// assert_eq!(means.as_str(), "AD");
/// assert_eq!(means.to_string(), "AD");
/// ```
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Means(String);

macro_rules! impl_axis_type {
    ($name:ident, $len:literal, $axis:ident) => {
        impl $name {
            /// Returns the canonical uppercase string form of this code.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0).finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IchiParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                parse_axis(s, $len, Axis::$axis).map($name)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IchiParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.collect_str(self)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

impl_axis_type!(Target, 3, Target);
impl_axis_type!(Action, 2, Action);
impl_axis_type!(Means, 2, Means);

impl Target {
    /// Returns the [`Section`] that this target's leading character(s)
    /// suggest it belongs to, if the crate is confident enough in the
    /// mapping to report one.
    ///
    /// **This is currently always `None`.** See the [`Section`] type
    /// documentation for why: this crate could not ground a specific
    /// leading-character range table against the published ICHI Beta-3
    /// tabular list with enough confidence to assert it as fact, and
    /// prefers returning `None` honestly over asserting a table that might
    /// be wrong.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_ichi::Target;
    ///
    /// let target: Target = "KAB".parse().unwrap();
    /// assert_eq!(target.section(), None);
    /// ```
    pub fn section(&self) -> Option<Section> {
        crate::section::section_for_target(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Target --------------------------------------------------------

    #[test]
    fn target_accepts_valid_codes() {
        for s in ["KAB", "AAA", "000", "Z9Z"] {
            let target: Target = s.parse().unwrap();
            assert_eq!(target.as_str(), s);
        }
    }

    #[test]
    fn target_canonicalizes_lowercase_input() {
        let target: Target = "kab".parse().unwrap();
        assert_eq!(target.as_str(), "KAB");
        assert_eq!(target.to_string(), "KAB");
    }

    #[test]
    fn target_rejects_empty() {
        let err = "".parse::<Target>().unwrap_err();
        assert_eq!(
            err,
            IchiParseError::Empty {
                axis: Some(Axis::Target)
            }
        );
    }

    #[test]
    fn target_rejects_wrong_length() {
        assert_eq!(
            "KA".parse::<Target>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "3",
                found: 2,
                axis: Some(Axis::Target)
            }
        );
        assert_eq!(
            "KABC".parse::<Target>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "3",
                found: 4,
                axis: Some(Axis::Target)
            }
        );
    }

    #[test]
    fn target_rejects_invalid_character() {
        assert_eq!(
            "K.B".parse::<Target>().unwrap_err(),
            IchiParseError::InvalidCharacter {
                position: 1,
                found: '.',
                axis: Some(Axis::Target)
            }
        );
    }

    #[test]
    fn target_try_from_matches_from_str() {
        assert_eq!(
            Target::try_from("kab").unwrap(),
            "kab".parse::<Target>().unwrap()
        );
    }

    #[test]
    fn target_section_is_none() {
        let target: Target = "KAB".parse().unwrap();
        assert_eq!(target.section(), None);
    }

    // -- Action ----------------------------------------------------------

    #[test]
    fn action_accepts_valid_codes() {
        for s in ["DB", "AA", "9Z"] {
            let action: Action = s.parse().unwrap();
            assert_eq!(action.as_str(), s);
        }
    }

    #[test]
    fn action_canonicalizes_lowercase_input() {
        let action: Action = "db".parse().unwrap();
        assert_eq!(action.as_str(), "DB");
    }

    #[test]
    fn action_rejects_empty() {
        assert_eq!(
            "".parse::<Action>().unwrap_err(),
            IchiParseError::Empty {
                axis: Some(Axis::Action)
            }
        );
    }

    #[test]
    fn action_rejects_wrong_length() {
        assert_eq!(
            "D".parse::<Action>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "2",
                found: 1,
                axis: Some(Axis::Action)
            }
        );
    }

    #[test]
    fn action_rejects_invalid_character() {
        assert_eq!(
            "D!".parse::<Action>().unwrap_err(),
            IchiParseError::InvalidCharacter {
                position: 1,
                found: '!',
                axis: Some(Axis::Action)
            }
        );
    }

    // -- Means -------------------------------------------------------------

    #[test]
    fn means_accepts_valid_codes() {
        for s in ["AD", "ZZ", "00"] {
            let means: Means = s.parse().unwrap();
            assert_eq!(means.as_str(), s);
        }
    }

    #[test]
    fn means_canonicalizes_lowercase_input() {
        let means: Means = "ad".parse().unwrap();
        assert_eq!(means.as_str(), "AD");
    }

    #[test]
    fn means_rejects_empty() {
        assert_eq!(
            "".parse::<Means>().unwrap_err(),
            IchiParseError::Empty {
                axis: Some(Axis::Means)
            }
        );
    }

    #[test]
    fn means_rejects_wrong_length() {
        assert_eq!(
            "ADD".parse::<Means>().unwrap_err(),
            IchiParseError::InvalidLength {
                expected: "2",
                found: 3,
                axis: Some(Axis::Means)
            }
        );
    }

    #[test]
    fn means_rejects_invalid_character() {
        assert_eq!(
            " D".parse::<Means>().unwrap_err(),
            IchiParseError::InvalidCharacter {
                position: 0,
                found: ' ',
                axis: Some(Axis::Means)
            }
        );
    }

    // -- ordering / trait set ----------------------------------------------

    #[test]
    fn axis_types_order_lexicographically() {
        let a: Target = "AAA".parse().unwrap();
        let b: Target = "AAB".parse().unwrap();
        assert!(a < b);
    }

    #[test]
    fn axis_types_are_hashable() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert("KAB".parse::<Target>().unwrap());
        set.insert("kab".parse::<Target>().unwrap());
        assert_eq!(set.len(), 1);
    }
}
