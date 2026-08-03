//! [`Section`]: the top-level grouping of ICHI interventions by the kind of
//! [`Target`](crate::Target) they act on.

use crate::axis::Target;

/// The top-level grouping of ICHI interventions by what kind of
/// [`Target`](crate::Target) the intervention acts on.
///
/// Per the WHO ICHI Reference Guide, interventions are grouped into chapters
/// which are in turn grouped into sections based on the Target of the
/// intervention. As a structural fact (not vendored classification
/// content), the chapter-number groupings are:
///
/// - Interventions on Body Systems and Functions (chapters 1-12)
/// - Interventions on Activities and Participation Domains (chapters 13-21)
/// - Interventions on the Environment (chapters 22-27, or 22-26 depending on
///   edition) and Health-related Behaviours (a further chapter in some
///   editions)
///
/// This enum mirrors the three sections named in the `who-fic-ichi` spec.
///
/// **Section detection is best-effort and may be unavailable.** ICHI is
/// still in beta (Beta-3), and while the *chapter-count* groupings above are
/// documented in WHO's reference material, this crate was not able to
/// ground a specific, verified mapping from a target code's **leading
/// character(s)** to its chapter/section against the official ICHI Beta-3
/// tabular list (that table is part of the licensed classification content
/// this repository does not vendor, and secondary sources were not
/// sufficient to reconstruct it with confidence — see notes in
/// `specs/who-fic-ichi.md`). Rather than assert a leading-character range
/// table that has not been verified against WHO's tabular list — which
/// would be actively misleading if wrong — [`Target::section`] and
/// [`crate::IchiCode::section`] currently always return `None`.
///
/// If a verified range table becomes available (e.g. from a future
/// data-loader subcrate that consumes WHO-licensed data), `section()` can
/// be upgraded to return `Some` for recognized ranges without changing this
/// type's shape.
///
/// # Examples
///
/// ```
/// use who_fic_ichi::Section;
///
/// // Section is non_exhaustive and PartialEq-comparable.
/// assert_eq!(Section::Environment, Section::Environment);
/// assert_ne!(Section::Environment, Section::BodySystemsAndFunctions);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Section {
    /// Interventions on body systems and functions (anatomy, physiology).
    BodySystemsAndFunctions,
    /// Interventions on activities and participation domains.
    ActivitiesAndParticipationDomains,
    /// Interventions on the environment (assistive products, support and
    /// relationships, and similar environmental factors).
    Environment,
}

/// Best-effort section lookup for a [`Target`]'s leading character(s).
///
/// See the [`Section`] documentation for why this currently always returns
/// `None`.
pub(crate) fn section_for_target(_target: &Target) -> Option<Section> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `section_for_target` currently returns `None` unconditionally,
    /// regardless of the target's leading character(s) — including for
    /// targets that plausibly fall in each of the three sections this
    /// crate would otherwise try to distinguish. These "known-good and
    /// known-bad range" boundary tests document that behavior: there is no
    /// boundary yet, only a documented `None`.
    #[test]
    fn section_for_target_is_always_none() {
        for s in ["AAA", "KAB", "VBA", "XZZ", "000", "ZZZ"] {
            let target: Target = s.parse().unwrap();
            assert_eq!(
                section_for_target(&target),
                None,
                "target {s} unexpectedly resolved to a section"
            );
        }
    }

    #[test]
    fn section_variants_are_distinct() {
        assert_ne!(
            Section::BodySystemsAndFunctions,
            Section::ActivitiesAndParticipationDomains
        );
        assert_ne!(
            Section::ActivitiesAndParticipationDomains,
            Section::Environment
        );
        assert_ne!(Section::BodySystemsAndFunctions, Section::Environment);
    }
}
