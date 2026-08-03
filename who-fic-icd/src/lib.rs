//! World Health Organization (WHO) International Classification of
//! Diseases (ICD).
//!
//! Part of the WHO Family of International Classifications (FIC).
//! See <https://www.who.int/standards/classifications/classification-of-diseases>.
//!
//! Planned modules — see `specs/who-fic-icd.md` in the repository:
//!
//! - `icd10` — ICD-10 codes (e.g. `I63.9`), chapters I–XXII
//! - `icd11` — ICD-11 MMS codes (e.g. `8B20`, `CA40.0`), extension codes,
//!   postcoordination clusters

/// The classification this crate implements.
pub const CLASSIFICATION: &str = "ICD";
