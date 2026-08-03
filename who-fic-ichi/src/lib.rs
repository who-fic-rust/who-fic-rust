//! World Health Organization (WHO) International Classification of
//! Health Interventions (ICHI).
//!
//! Part of the WHO Family of International Classifications (FIC).
//! See <https://www.who.int/standards/classifications/international-classification-of-health-interventions>.
//!
//! Planned API — see `specs/who-fic-ichi.md` in the repository:
//!
//! - `Target`, `Action`, `Means` — the three ICHI axes
//! - `IchiCode` — composed `TTT.AA.MM` intervention codes
//!   (e.g. `KAB.DB.AD`)
//! - `Section` — body systems & functions / activities & participation
//!   domains / environment
//!
//! Note: ICHI's latest public release is a beta; details may change before
//! final adoption.

/// The classification this crate implements.
pub const CLASSIFICATION: &str = "ICHI";
