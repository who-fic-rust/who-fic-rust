//! Parse error type for ICHI codes, and the [`Axis`] discriminant used to
//! report which of the three ICHI axes a parse failure came from.

use std::fmt;

/// Identifies one of the three ICHI axes: [`Target`](crate::Target),
/// [`Action`](crate::Action), or [`Means`](crate::Means).
///
/// Used by [`IchiParseError`] to report which axis of a dotted
/// `TARGET.ACTION.MEANS` code failed to parse.
///
/// # Examples
///
/// ```
/// use who_fic_ichi::Axis;
///
/// assert_eq!(Axis::Target.to_string(), "target");
/// assert_ne!(Axis::Action, Axis::Means);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Axis {
    /// The [`Target`](crate::Target) axis: the entity on which the action is
    /// carried out.
    Target,
    /// The [`Action`](crate::Action) axis: the deed done by the actor to the
    /// target.
    Action,
    /// The [`Means`](crate::Means) axis: the processes and methods by which
    /// the action is carried out.
    Means,
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Axis::Target => "target",
            Axis::Action => "action",
            Axis::Means => "means",
        })
    }
}

/// Error produced when parsing an ICHI [`Target`](crate::Target),
/// [`Action`](crate::Action), [`Means`](crate::Means), or composed
/// [`IchiCode`](crate::IchiCode) fails.
///
/// Follows the shared error shape used across the `who-fic` crates (empty
/// input / invalid length / invalid character / invalid structure), with an
/// additional `axis` field on every variant identifying which of the three
/// ICHI axes the failure belongs to, when known.
///
/// `axis` is `Some(_)` whenever the failure occurred while validating a
/// specific `TARGET`, `ACTION`, or `MEANS` segment (whether that segment was
/// parsed standalone or as part of a dotted [`IchiCode`](crate::IchiCode)).
/// It is `None` for failures that concern the composed code's overall
/// structure before any individual segment is examined (an empty whole
/// input, or the wrong number of dot-separated segments).
///
/// # Examples
///
/// ```
/// use who_fic_ichi::{Axis, IchiCode, IchiParseError};
///
/// let err: IchiParseError = "KA.DB.AD".parse::<IchiCode>().unwrap_err();
/// assert_eq!(
///     err,
///     IchiParseError::InvalidLength { expected: "3", found: 2, axis: Some(Axis::Target) }
/// );
/// assert!(err.to_string().contains("target"));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IchiParseError {
    /// The input was empty.
    Empty {
        /// The axis being parsed, if the empty input is a single segment of
        /// a larger dotted code; `None` if the whole code input was empty.
        axis: Option<Axis>,
    },
    /// The input (or one of its dot-separated segments) had the wrong
    /// number of characters.
    InvalidLength {
        /// A human-readable description of the expected length, e.g. `"3"`.
        expected: &'static str,
        /// The number of characters actually found.
        found: usize,
        /// The axis whose segment had the wrong length, if known.
        axis: Option<Axis>,
    },
    /// A character outside the allowed alphabet (`[A-Za-z0-9]`) was found.
    InvalidCharacter {
        /// The zero-based character position within the failing axis
        /// segment (not the whole dotted code).
        position: usize,
        /// The offending character.
        found: char,
        /// The axis whose segment contained the invalid character, if
        /// known.
        axis: Option<Axis>,
    },
    /// The overall dotted-code structure was invalid (wrong separators,
    /// wrong number of segments) independent of any single segment's
    /// content.
    InvalidStructure {
        /// A human-readable explanation.
        reason: &'static str,
        /// The axis this structural problem pertains to, if known.
        axis: Option<Axis>,
    },
}

impl fmt::Display for IchiParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IchiParseError::Empty { axis: Some(axis) } => {
                write!(f, "empty input for {axis} axis")
            }
            IchiParseError::Empty { axis: None } => write!(f, "empty input"),
            IchiParseError::InvalidLength {
                expected,
                found,
                axis: Some(axis),
            } => write!(
                f,
                "invalid length for {axis} axis: expected {expected} character(s), found {found}"
            ),
            IchiParseError::InvalidLength {
                expected,
                found,
                axis: None,
            } => write!(
                f,
                "invalid length: expected {expected} character(s), found {found}"
            ),
            IchiParseError::InvalidCharacter {
                position,
                found,
                axis: Some(axis),
            } => write!(
                f,
                "invalid character {found:?} at position {position} in {axis} axis"
            ),
            IchiParseError::InvalidCharacter {
                position,
                found,
                axis: None,
            } => write!(f, "invalid character {found:?} at position {position}"),
            IchiParseError::InvalidStructure {
                reason,
                axis: Some(axis),
            } => write!(f, "invalid structure in {axis} axis: {reason}"),
            IchiParseError::InvalidStructure { reason, axis: None } => {
                write!(f, "invalid structure: {reason}")
            }
        }
    }
}

impl std::error::Error for IchiParseError {}
