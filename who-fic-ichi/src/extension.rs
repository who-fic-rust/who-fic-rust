//! Design sketch for ICHI **extension codes** (deferred, not implemented).
//!
//! ICHI defines extension codes that refine an [`IchiCode`](crate::IchiCode)
//! with post-coordinated detail — conceptually similar to ICD-11 extension
//! codes. Per the WHO ICHI Reference Guide, extension codes cover things
//! like:
//!
//! - **Laterality** (left / right / bilateral / unspecified)
//! - **Quantity** (e.g. number of vessels, teeth, lesions treated)
//! - **Anatomy detail** (a more specific anatomical site than the
//!   [`Target`](crate::Target) axis alone expresses)
//! - **Therapeutic and assistive products**
//! - **Medicaments**
//! - **Essential laboratory tests**
//! - **Telehealth** modifiers
//!
//! Where applicable, ICHI reuses the same extension codes as ICD-11, and
//! ICF categories may themselves be used as extension codes to add detail
//! to functioning-related targets. Extension codes are applied
//! *post-coordination*: they attach to a base [`IchiCode`](crate::IchiCode)
//! rather than being embedded in its 9-character canonical form, so a
//! future design would most likely represent an extended code as a struct
//! pairing an [`IchiCode`](crate::IchiCode) with a collection of typed or
//! stringly-typed extension values (grouped by extension category), plus
//! parsing/formatting for however WHO ultimately publishes the
//! post-coordination syntax.
//!
//! **Implementation is deferred.** The post-beta shape of extension codes
//! (their own code syntax, the set of extension categories, and how they
//! serialize alongside a stem code) is not yet stable enough in ICHI
//! Beta-3 to commit to a concrete API. This module intentionally contains
//! no types yet; it exists as a placeholder and a home for this design
//! note. See the backlog in `tasks.md` in the repository root.
