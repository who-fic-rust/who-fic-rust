//! World Health Organization (WHO) International Classification of
//! Functioning, Disability and Health (ICF).
//!
//! Part of the WHO Family of International Classifications (FIC).
//! See <https://www.who.int/standards/classifications/international-classification-of-functioning-disability-and-health>.
//!
//! ICF codes have three parts: a component letter ([`Component`]), a
//! numeric hierarchy ([`IcfCode`]), and optional qualifiers
//! ([`QualifiedIcfCode`]). See `specs/who-fic-icf.md` in the repository
//! for the full grammar.
//!
//! # Example
//!
//! ```
//! use std::str::FromStr;
//! use who_fic_icf::{Component, IcfCode, QualifiedIcfCode};
//!
//! // The bare hierarchy code, as it appears in the tabulation.
//! let code = IcfCode::from_str("b280").unwrap();
//! assert_eq!(code.component(), Component::BodyFunctions);
//! assert_eq!(code.parent().unwrap().as_str(), "b2");
//!
//! // The same code with an assessment qualifier attached.
//! let qualified = QualifiedIcfCode::from_str("b280.2").unwrap();
//! assert_eq!(qualified.code(), &code);
//! ```

#![warn(missing_docs)]

mod code;
mod component;
mod error;
mod level;
#[cfg(feature = "linearization")]
pub mod linearization;
mod qualified;
mod qualifier;

pub use code::IcfCode;
pub use component::Component;
pub use error::IcfParseError;
pub use level::Level;
pub use qualified::{
    ActivitiesQualifiers, BodyStructureQualifiers, EnvironmentalQualifier, QualifiedIcfCode,
    QualifierPayload,
};
pub use qualifier::Qualifier;

/// The classification this crate implements.
pub const CLASSIFICATION: &str = "ICF";

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn classification_constant() {
        assert_eq!(CLASSIFICATION, "ICF");
    }

    #[test]
    fn public_types_resolve_at_crate_root() {
        let _: IcfCode = IcfCode::from_str("b280").unwrap();
        let _: Component = Component::BodyFunctions;
        let _: Qualifier = Qualifier::Moderate;
        let _: QualifiedIcfCode = QualifiedIcfCode::from_str("b280.2").unwrap();
        let _: IcfParseError = IcfParseError::Empty;
        let _: Level = Level::Chapter;
    }
}
