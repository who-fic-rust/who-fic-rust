/// The hierarchy depth of an [`IcfCode`](crate::IcfCode).
///
/// ICF codes have four levels, distinguished by the number of digits after
/// the component letter: 1 digit (chapter), 3 digits (second level), 4
/// digits (third level), or 5 digits (fourth level). There is no 2-digit
/// level.
///
/// Ordered from shallowest to deepest, so `Level::Chapter < Level::FourthLevel`.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use who_fic_icf::{IcfCode, Level};
///
/// assert_eq!(IcfCode::from_str("b280").unwrap().level(), Level::SecondLevel);
/// assert!(Level::Chapter < Level::FourthLevel);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Level {
    /// 1 digit, e.g. `b2`.
    Chapter,
    /// 3 digits, e.g. `b280`.
    SecondLevel,
    /// 4 digits, e.g. `b2801`.
    ThirdLevel,
    /// 5 digits, e.g. `b28010`.
    FourthLevel,
}
