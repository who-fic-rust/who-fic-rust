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
//! Shared vocabulary (the `Classification` enum and the `FicError` parse
//! error) is planned here — see `specs/who-fic.md` in the repository.

#[cfg(feature = "icd")]
pub use who_fic_icd as icd;

#[cfg(feature = "icf")]
pub use who_fic_icf as icf;

#[cfg(feature = "ichi")]
pub use who_fic_ichi as ichi;

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "icd")]
    fn icd_reexport_resolves() {
        assert_eq!(crate::icd::CLASSIFICATION, "ICD");
    }

    #[test]
    #[cfg(feature = "icf")]
    fn icf_reexport_resolves() {
        assert_eq!(crate::icf::CLASSIFICATION, "ICF");
    }

    #[test]
    #[cfg(feature = "ichi")]
    fn ichi_reexport_resolves() {
        assert_eq!(crate::ichi::CLASSIFICATION, "ICHI");
    }
}
