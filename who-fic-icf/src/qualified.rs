use crate::{Component, IcfCode, IcfParseError, Qualifier};
use std::fmt;
use std::str::FromStr;

/// Positional qualifiers for a **body structures** (`s`) code: extent of
/// impairment, nature of change, and location, in that order. 1 to 3 are
/// present, always filled from the front (there is never a `nature` or
/// `location` without the positions before it being present).
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::{Qualifier, QualifiedIcfCode};
///
/// let q = QualifiedIcfCode::from_str("s730.312").unwrap();
/// let s = q.qualifiers().as_body_structures().unwrap();
/// assert_eq!(s.extent(), Qualifier::Severe);
/// assert_eq!(s.nature(), Some(Qualifier::Mild));
/// assert_eq!(s.location(), Some(Qualifier::Moderate));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BodyStructureQualifiers {
    extent: Qualifier,
    nature: Option<Qualifier>,
    location: Option<Qualifier>,
}

impl BodyStructureQualifiers {
    /// The first positional qualifier: extent of impairment. Always
    /// present.
    pub fn extent(&self) -> Qualifier {
        self.extent
    }

    /// The second positional qualifier: nature of change, if present.
    pub fn nature(&self) -> Option<Qualifier> {
        self.nature
    }

    /// The third positional qualifier: location, if present.
    pub fn location(&self) -> Option<Qualifier> {
        self.location
    }
}

/// Qualifiers for an **activities and participation** (`d`) code:
/// performance then capacity, in that order, with two further positions
/// reserved by the ICF for optional future use.
///
/// Standard use fills exactly `performance` and `capacity` (e.g.
/// `d450.12`); `position_3` and `position_4` are parsed if present but are
/// not part of the current WHO standard.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::{Qualifier, QualifiedIcfCode};
///
/// let q = QualifiedIcfCode::from_str("d450.12").unwrap();
/// let d = q.qualifiers().as_activities_and_participation().unwrap();
/// assert_eq!(d.performance(), Qualifier::Mild);
/// assert_eq!(d.capacity(), Some(Qualifier::Moderate));
/// assert_eq!(d.position_3(), None);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActivitiesQualifiers {
    performance: Qualifier,
    capacity: Option<Qualifier>,
    position_3: Option<Qualifier>,
    position_4: Option<Qualifier>,
}

impl ActivitiesQualifiers {
    /// The first positional qualifier: performance (in the person's actual
    /// environment). Always present.
    pub fn performance(&self) -> Qualifier {
        self.performance
    }

    /// The second positional qualifier: capacity (without assistance), if
    /// present. Standard ICF use always includes this alongside
    /// `performance`.
    pub fn capacity(&self) -> Option<Qualifier> {
        self.capacity
    }

    /// The third positional qualifier, reserved by the ICF for optional
    /// future use.
    pub fn position_3(&self) -> Option<Qualifier> {
        self.position_3
    }

    /// The fourth positional qualifier, reserved by the ICF for optional
    /// future use.
    pub fn position_4(&self) -> Option<Qualifier> {
        self.position_4
    }
}

/// The qualifier for an **environmental factors** (`e`) code, which is
/// either a barrier (separator `.`) or a facilitator (separator `+`).
///
/// The barrier/facilitator distinction is carried by the separator
/// character used in the code, so it is modeled as part of the value
/// itself rather than as a side channel.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::{EnvironmentalQualifier, Qualifier, QualifiedIcfCode};
///
/// let barrier = QualifiedIcfCode::from_str("e150.2").unwrap();
/// assert_eq!(
///     barrier.qualifiers().as_environmental_factors(),
///     Some(EnvironmentalQualifier::Barrier(Qualifier::Moderate))
/// );
///
/// let facilitator = QualifiedIcfCode::from_str("e150+2").unwrap();
/// assert_eq!(
///     facilitator.qualifiers().as_environmental_factors(),
///     Some(EnvironmentalQualifier::Facilitator(Qualifier::Moderate))
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EnvironmentalQualifier {
    /// A barrier (separator `.`), e.g. `e150.2`.
    Barrier(Qualifier),
    /// A facilitator (separator `+`), e.g. `e150+2`.
    Facilitator(Qualifier),
}

impl EnvironmentalQualifier {
    /// The underlying generic-scale qualifier, regardless of whether this
    /// is a barrier or a facilitator.
    pub fn qualifier(&self) -> Qualifier {
        match self {
            EnvironmentalQualifier::Barrier(q) => *q,
            EnvironmentalQualifier::Facilitator(q) => *q,
        }
    }

    /// Whether this is a barrier.
    pub fn is_barrier(&self) -> bool {
        matches!(self, EnvironmentalQualifier::Barrier(_))
    }

    /// Whether this is a facilitator.
    pub fn is_facilitator(&self) -> bool {
        matches!(self, EnvironmentalQualifier::Facilitator(_))
    }

    /// The separator character this qualifier is written with: `.` for a
    /// barrier, `+` for a facilitator.
    pub fn separator(&self) -> char {
        match self {
            EnvironmentalQualifier::Barrier(_) => '.',
            EnvironmentalQualifier::Facilitator(_) => '+',
        }
    }
}

/// The component-specific qualifier payload of a [`QualifiedIcfCode`].
///
/// This is the type-safety core of this crate: each ICF component pairs
/// with exactly one payload shape (a single qualifier for body functions,
/// up to three positional qualifiers for body structures, performance and
/// capacity for activities and participation, or a barrier/facilitator
/// qualifier for environmental factors). Because the only way to build a
/// `QualifiedIcfCode` is by parsing, a payload can never end up attached
/// to the wrong component — e.g. a `d` code with a `+` separator, or a `b`
/// code with two qualifier digits, is a parse error rather than a
/// constructible value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QualifierPayload {
    /// Body functions (`b`): a single generic-scale qualifier.
    BodyFunctions(Qualifier),
    /// Body structures (`s`): 1 to 3 positional qualifiers.
    BodyStructures(BodyStructureQualifiers),
    /// Activities and participation (`d`): performance, capacity, and up
    /// to two further optional-use positions.
    ActivitiesAndParticipation(ActivitiesQualifiers),
    /// Environmental factors (`e`): a barrier or facilitator qualifier.
    EnvironmentalFactors(EnvironmentalQualifier),
}

impl QualifierPayload {
    /// The payload as a body-functions qualifier, if this is one.
    pub fn as_body_functions(&self) -> Option<Qualifier> {
        match self {
            QualifierPayload::BodyFunctions(q) => Some(*q),
            _ => None,
        }
    }

    /// The payload as body-structure qualifiers, if this is one.
    pub fn as_body_structures(&self) -> Option<BodyStructureQualifiers> {
        match self {
            QualifierPayload::BodyStructures(q) => Some(*q),
            _ => None,
        }
    }

    /// The payload as activities-and-participation qualifiers, if this is
    /// one.
    pub fn as_activities_and_participation(&self) -> Option<ActivitiesQualifiers> {
        match self {
            QualifierPayload::ActivitiesAndParticipation(q) => Some(*q),
            _ => None,
        }
    }

    /// The payload as an environmental-factors qualifier, if this is one.
    pub fn as_environmental_factors(&self) -> Option<EnvironmentalQualifier> {
        match self {
            QualifierPayload::EnvironmentalFactors(q) => Some(*q),
            _ => None,
        }
    }
}

/// An ICF hierarchy code together with its component-appropriate
/// qualifier(s), e.g. `b280.2`, `s730.312`, `d450.12`, `e150+2`.
///
/// # Representation
///
/// `QualifiedIcfCode` wraps the underlying [`IcfCode`] plus a
/// [`QualifierPayload`] whose shape is tied to that code's [`Component`]
/// (see `QualifierPayload` for why this makes illegal combinations
/// unrepresentable), plus a cached canonical string used by
/// [`as_str`](QualifiedIcfCode::as_str) and [`Display`](std::fmt::Display).
/// The only public way to construct a value is [`FromStr`]/[`TryFrom<&str>`], which
/// validates the qualifier structure against the code's component, so an
/// existing `QualifiedIcfCode` is always internally consistent.
///
/// A bare [`IcfCode`] with no qualifiers remains separately valid — it is
/// what appears in the tabulation itself — `QualifiedIcfCode` is additive,
/// not a replacement.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::QualifiedIcfCode;
///
/// let q = QualifiedIcfCode::from_str("b280.2").unwrap();
/// assert_eq!(q.as_str(), "b280.2");
/// assert_eq!(q.code().as_str(), "b280");
/// ```
#[derive(Debug, Clone)]
pub struct QualifiedIcfCode {
    code: IcfCode,
    payload: QualifierPayload,
    repr: String,
}

impl QualifiedIcfCode {
    /// The underlying unqualified code.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::QualifiedIcfCode;
    ///
    /// let q = QualifiedIcfCode::from_str("b280.2").unwrap();
    /// assert_eq!(q.code().as_str(), "b280");
    /// ```
    pub fn code(&self) -> &IcfCode {
        &self.code
    }

    /// The component of the underlying code.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::{Component, QualifiedIcfCode};
    ///
    /// let q = QualifiedIcfCode::from_str("e150+2").unwrap();
    /// assert_eq!(q.component(), Component::EnvironmentalFactors);
    /// ```
    pub fn component(&self) -> Component {
        self.code.component()
    }

    /// The component-appropriate qualifier payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::{Qualifier, QualifiedIcfCode};
    ///
    /// let q = QualifiedIcfCode::from_str("b280.2").unwrap();
    /// assert_eq!(q.qualifiers().as_body_functions(), Some(Qualifier::Moderate));
    /// ```
    pub fn qualifiers(&self) -> QualifierPayload {
        self.payload
    }

    /// The canonical string form of this qualified code.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::QualifiedIcfCode;
    ///
    /// assert_eq!(QualifiedIcfCode::from_str("s730.312").unwrap().as_str(), "s730.312");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl fmt::Display for QualifiedIcfCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.repr)
    }
}

impl PartialEq for QualifiedIcfCode {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.payload == other.payload
    }
}

impl Eq for QualifiedIcfCode {}

impl std::hash::Hash for QualifiedIcfCode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state);
        self.payload.hash(state);
    }
}

impl PartialOrd for QualifiedIcfCode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QualifiedIcfCode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.code
            .cmp(&other.code)
            .then_with(|| self.payload.cmp(&other.payload))
    }
}

/// Parses a run of ASCII-digit characters (an already-sliced suffix) into
/// qualifiers, returning them positionally. `base_position` is the
/// absolute character position of `suffix[0]` in the original input, used
/// to report accurate error positions.
fn parse_qualifier_digits(
    suffix: &str,
    base_position: usize,
) -> Result<Vec<Qualifier>, IcfParseError> {
    suffix
        .chars()
        .enumerate()
        .map(|(i, c)| {
            Qualifier::try_from(c).map_err(|_| IcfParseError::InvalidCharacter {
                position: base_position + i,
                found: c,
            })
        })
        .collect()
}

impl FromStr for QualifiedIcfCode {
    type Err = IcfParseError;

    /// Parses a qualified ICF code.
    ///
    /// The code part before the separator is parsed as an [`IcfCode`];
    /// the separator (`.` or `+`) and the digits after it are then
    /// validated against that code's component:
    ///
    /// - `b`: separator must be `.`, exactly 1 qualifier digit.
    /// - `s`: separator must be `.`, 1 to 3 qualifier digits.
    /// - `d`: separator must be `.`, 1 to 4 qualifier digits (2 — performance
    ///   and capacity — is standard use).
    /// - `e`: separator is `.` (barrier) or `+` (facilitator), exactly 1
    ///   qualifier digit.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use who_fic_icf::{IcfParseError, QualifiedIcfCode};
    ///
    /// assert!(QualifiedIcfCode::from_str("b280.2").is_ok());
    /// assert!(QualifiedIcfCode::from_str("b280+2").is_err());
    /// assert_eq!(
    ///     QualifiedIcfCode::from_str(""),
    ///     Err(IcfParseError::Empty)
    /// );
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(IcfParseError::Empty);
        }

        let sep_index = s.find(['.', '+']).ok_or(IcfParseError::InvalidStructure {
            reason: "missing qualifier separator '.' or '+'",
        })?;
        let separator = s[sep_index..].chars().next().unwrap();

        let code = IcfCode::from_str(&s[..sep_index])?;
        let component = code.component();
        let suffix = &s[sep_index + separator.len_utf8()..];
        let suffix_start = sep_index + separator.len_utf8();

        if separator == '+' && component != Component::EnvironmentalFactors {
            return Err(IcfParseError::InvalidCharacter {
                position: sep_index,
                found: '+',
            });
        }

        let payload = match component {
            Component::BodyFunctions => {
                if suffix.chars().count() != 1 {
                    return Err(IcfParseError::InvalidLength {
                        expected: "1 qualifier digit",
                        found: suffix.chars().count(),
                    });
                }
                let qualifiers = parse_qualifier_digits(suffix, suffix_start)?;
                QualifierPayload::BodyFunctions(qualifiers[0])
            }
            Component::BodyStructures => {
                let n = suffix.chars().count();
                if !(1..=3).contains(&n) {
                    return Err(IcfParseError::InvalidLength {
                        expected: "1 to 3 qualifier digits",
                        found: n,
                    });
                }
                let qualifiers = parse_qualifier_digits(suffix, suffix_start)?;
                QualifierPayload::BodyStructures(BodyStructureQualifiers {
                    extent: qualifiers[0],
                    nature: qualifiers.get(1).copied(),
                    location: qualifiers.get(2).copied(),
                })
            }
            Component::ActivitiesAndParticipation => {
                let n = suffix.chars().count();
                if !(1..=4).contains(&n) {
                    return Err(IcfParseError::InvalidLength {
                        expected: "1 to 4 qualifier digits",
                        found: n,
                    });
                }
                let qualifiers = parse_qualifier_digits(suffix, suffix_start)?;
                QualifierPayload::ActivitiesAndParticipation(ActivitiesQualifiers {
                    performance: qualifiers[0],
                    capacity: qualifiers.get(1).copied(),
                    position_3: qualifiers.get(2).copied(),
                    position_4: qualifiers.get(3).copied(),
                })
            }
            Component::EnvironmentalFactors => {
                if suffix.chars().count() != 1 {
                    return Err(IcfParseError::InvalidLength {
                        expected: "1 qualifier digit",
                        found: suffix.chars().count(),
                    });
                }
                let qualifiers = parse_qualifier_digits(suffix, suffix_start)?;
                let q = qualifiers[0];
                let eq = if separator == '+' {
                    EnvironmentalQualifier::Facilitator(q)
                } else {
                    EnvironmentalQualifier::Barrier(q)
                };
                QualifierPayload::EnvironmentalFactors(eq)
            }
        };

        let mut repr = String::with_capacity(code.as_str().len() + 1 + suffix.len());
        repr.push_str(code.as_str());
        repr.push(separator);
        repr.push_str(suffix);

        Ok(QualifiedIcfCode {
            code,
            payload,
            repr,
        })
    }
}

impl TryFrom<&str> for QualifiedIcfCode {
    type Error = IcfParseError;

    /// Parses a qualified ICF code. See `QualifiedIcfCode::from_str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use who_fic_icf::QualifiedIcfCode;
    ///
    /// assert!(QualifiedIcfCode::try_from("d450.12").is_ok());
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Serializes as the canonical string form (via [`Display`](fmt::Display)),
/// not as a struct.
#[cfg(feature = "serde")]
impl serde::Serialize for QualifiedIcfCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Deserializes from the canonical string form (via [`FromStr`]).
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for QualifiedIcfCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCEPT: &[&str] = &[
        "b280.2", "s730.312", "s730.3", "d450.12", "d450.1", "e150+2", "e150.2",
    ];

    #[test]
    fn accepts_valid_qualified_codes() {
        for s in ACCEPT {
            assert!(
                QualifiedIcfCode::from_str(s).is_ok(),
                "expected {s} to parse"
            );
        }
    }

    #[test]
    fn round_trip() {
        for s in ACCEPT {
            let q = QualifiedIcfCode::from_str(s).unwrap();
            assert_eq!(q.as_str(), *s);
            assert_eq!(QualifiedIcfCode::from_str(q.as_str()).unwrap(), q);
        }
    }

    #[test]
    fn body_functions_exactly_one_qualifier() {
        let q = QualifiedIcfCode::from_str("b280.2").unwrap();
        assert_eq!(
            q.qualifiers().as_body_functions(),
            Some(Qualifier::Moderate)
        );

        assert!(matches!(
            QualifiedIcfCode::from_str("b280.22"),
            Err(IcfParseError::InvalidLength { .. })
        ));
    }

    #[test]
    fn rejects_invalid_qualifier_digit() {
        assert_eq!(
            QualifiedIcfCode::from_str("b280.5"),
            Err(IcfParseError::InvalidCharacter {
                position: 5,
                found: '5'
            })
        );
    }

    #[test]
    fn rejects_facilitator_on_non_environmental_component() {
        assert_eq!(
            QualifiedIcfCode::from_str("b280+2"),
            Err(IcfParseError::InvalidCharacter {
                position: 4,
                found: '+'
            })
        );
    }

    #[test]
    fn rejects_overlong_qualifier_string() {
        assert!(matches!(
            QualifiedIcfCode::from_str("s730.3124"),
            Err(IcfParseError::InvalidLength { .. })
        ));
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(QualifiedIcfCode::from_str(""), Err(IcfParseError::Empty));
    }

    #[test]
    fn rejects_missing_separator() {
        assert_eq!(
            QualifiedIcfCode::from_str("b280"),
            Err(IcfParseError::InvalidStructure {
                reason: "missing qualifier separator '.' or '+'"
            })
        );
    }

    #[test]
    fn body_structures_positional_qualifiers() {
        let q = QualifiedIcfCode::from_str("s730.312").unwrap();
        let s = q.qualifiers().as_body_structures().unwrap();
        assert_eq!(s.extent(), Qualifier::Severe);
        assert_eq!(s.nature(), Some(Qualifier::Mild));
        assert_eq!(s.location(), Some(Qualifier::Moderate));

        let q1 = QualifiedIcfCode::from_str("s730.3").unwrap();
        let s1 = q1.qualifiers().as_body_structures().unwrap();
        assert_eq!(s1.extent(), Qualifier::Severe);
        assert_eq!(s1.nature(), None);
        assert_eq!(s1.location(), None);
    }

    #[test]
    fn activities_performance_and_capacity() {
        let q = QualifiedIcfCode::from_str("d450.12").unwrap();
        let d = q.qualifiers().as_activities_and_participation().unwrap();
        assert_eq!(d.performance(), Qualifier::Mild);
        assert_eq!(d.capacity(), Some(Qualifier::Moderate));
        assert_eq!(d.position_3(), None);
        assert_eq!(d.position_4(), None);
    }

    #[test]
    fn activities_allows_one_to_four_digits() {
        assert!(QualifiedIcfCode::from_str("d450.1").is_ok());
        assert!(QualifiedIcfCode::from_str("d450.123").is_ok());
        assert!(QualifiedIcfCode::from_str("d450.1234").is_ok());
        assert!(matches!(
            QualifiedIcfCode::from_str("d450.12345"),
            Err(IcfParseError::InvalidLength { .. })
        ));
    }

    #[test]
    fn environmental_barrier_vs_facilitator() {
        let barrier = QualifiedIcfCode::from_str("e150.2").unwrap();
        assert_eq!(
            barrier.qualifiers().as_environmental_factors(),
            Some(EnvironmentalQualifier::Barrier(Qualifier::Moderate))
        );

        let facilitator = QualifiedIcfCode::from_str("e150+2").unwrap();
        assert_eq!(
            facilitator.qualifiers().as_environmental_factors(),
            Some(EnvironmentalQualifier::Facilitator(Qualifier::Moderate))
        );

        assert_ne!(barrier, facilitator);
    }

    #[test]
    fn environmental_rejects_more_than_one_digit() {
        assert!(matches!(
            QualifiedIcfCode::from_str("e150.22"),
            Err(IcfParseError::InvalidLength { .. })
        ));
    }
}
