//! World Health Organization (WHO) International Classification of
//! Functioning, Disability and Health (ICF).
//!
//! Part of the WHO Family of International Classifications (FIC).
//! See <https://www.who.int/standards/classifications/international-classification-of-functioning-disability-and-health>.
//!
//! Planned API — see `specs/who-fic-icf.md` in the repository:
//!
//! - `Component` — body functions (`b`), body structures (`s`),
//!   activities and participation (`d`), environmental factors (`e`)
//! - `IcfCode` — hierarchical codes (e.g. `b280`, `s7301`) with level and
//!   parent navigation
//! - Qualifiers — generic scale 0–4/8/9, component-specific structure,
//!   environmental barrier (`.`) vs. facilitator (`+`)

/// The classification this crate implements.
pub const CLASSIFICATION: &str = "ICF";
