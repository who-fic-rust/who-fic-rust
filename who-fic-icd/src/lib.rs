//! World Health Organization (WHO) International Classification of
//! Diseases (ICD).
//!
//! Part of the WHO Family of International Classifications (FIC).
//! See <https://www.who.int/standards/classifications/classification-of-diseases>.
//!
//! Two revisions are in wide use and both are in scope, as separate modules
//! of this crate — see `specs/who-fic-icd.md` in the repository:
//!
//! - [`icd10`] — ICD-10 codes (e.g. `I63.9`), chapters I–XXII
//! - [`icd11`] — ICD-11 MMS codes (e.g. `8B20`, `CA40.0`), extension codes,
//!   postcoordination clusters
//!
//! ICD-10 and ICD-11 have different code syntaxes and deliberately do not
//! share a code type: [`icd10::Icd10Code`] and [`icd11::Icd11Code`] are
//! distinct types that cannot be confused for one another.
//!
//! These parsers validate *syntax and structure* only (allowed characters,
//! lengths, chapter ranges) — not *existence* (whether WHO has actually
//! assigned a given code). Existence checks require WHO data and are out of
//! scope for this crate.

#![warn(missing_docs)]

pub mod icd10;
pub mod icd11;

/// The classification this crate implements.
pub const CLASSIFICATION: &str = "ICD";
