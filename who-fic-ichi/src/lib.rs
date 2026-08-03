//! World Health Organization (WHO) International Classification of
//! Health Interventions (ICHI).
//!
//! Part of the WHO Family of International Classifications (FIC).
//! See <https://www.who.int/standards/classifications/international-classification-of-health-interventions>.
//!
//! **Stability note:** ICHI's latest public release is a **beta (Beta-3)**.
//! Its axis architecture (Target / Action / Means) is stable, but code
//! details may change before final adoption. Enums in this crate that
//! mirror WHO value sets ([`Section`], [`Axis`]) are `#[non_exhaustive]`
//! accordingly, and this crate validates code *syntax* only — it does not
//! assert that a given code has actually been assigned by WHO.
//!
//! # API
//!
//! - [`Target`], [`Action`], [`Means`] — the three ICHI axes, each a
//!   validated newtype over fixed-length uppercase-alphanumeric strings.
//!   These are public standalone types (the axes are published by WHO as
//!   independent value sets), not just internal fields of [`IchiCode`].
//! - [`IchiCode`] — the composed `TARGET.ACTION.MEANS` intervention code
//!   (e.g. `KAB.DB.AD`).
//! - [`Section`] — the best-effort grouping of a code's target into "body
//!   systems & functions" / "activities & participation domains" /
//!   "environment" (see that type's docs for important caveats).
//! - [`Axis`], [`IchiParseError`] — error reporting: which axis of a
//!   dotted code failed to parse, and why.
//! - [`extension`] — a documentation-only module sketching ICHI's
//!   post-coordinated extension codes (laterality, quantity, …), not yet
//!   implemented.
//! - [`linearization`] (optional `linearization` feature) — adapts rows
//!   from a WHO ICHI "Simplified Linearization Output" export (parsed by
//!   [`who_fic_linearization`]) into a lookup from [`IchiCode`] to title.
//!
//! # Example
//!
//! ```
//! use who_fic_ichi::IchiCode;
//!
//! let code: IchiCode = "kab.db.ad".parse().unwrap();
//! assert_eq!(code.to_string(), "KAB.DB.AD");
//! assert_eq!(code.target().as_str(), "KAB");
//! assert_eq!(code.action().as_str(), "DB");
//! assert_eq!(code.means().as_str(), "AD");
//! ```

mod axis;
mod code;
mod error;
mod section;

pub mod extension;
#[cfg(feature = "linearization")]
pub mod linearization;

pub use axis::{Action, Means, Target};
pub use code::IchiCode;
pub use error::{Axis, IchiParseError};
pub use section::Section;

/// The classification this crate implements.
///
/// # Examples
///
/// ```
/// assert_eq!(who_fic_ichi::CLASSIFICATION, "ICHI");
/// ```
pub const CLASSIFICATION: &str = "ICHI";
