//! Property tests: parse -> format -> parse round trips, and panic-safety
//! of the parsers on arbitrary input.

use proptest::prelude::*;
use std::str::FromStr;
use who_fic_ichi::{Action, IchiCode, Means, Target};

/// A single valid ICHI axis alphabet character.
fn alnum_char() -> impl Strategy<Value = char> {
    prop_oneof![
        (b'A'..=b'Z').prop_map(|b| b as char),
        (b'0'..=b'9').prop_map(|b| b as char),
    ]
}

fn valid_segment(len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(alnum_char(), len).prop_map(|cs| cs.into_iter().collect())
}

fn valid_target_string() -> impl Strategy<Value = String> {
    valid_segment(3)
}

fn valid_action_string() -> impl Strategy<Value = String> {
    valid_segment(2)
}

fn valid_means_string() -> impl Strategy<Value = String> {
    valid_segment(2)
}

fn valid_ichi_code_string() -> impl Strategy<Value = String> {
    (
        valid_target_string(),
        valid_action_string(),
        valid_means_string(),
    )
        .prop_map(|(t, a, m)| format!("{t}.{a}.{m}"))
}

proptest! {
    #[test]
    fn target_round_trips(s in valid_target_string()) {
        let target = Target::from_str(&s).unwrap();
        let formatted = target.to_string();
        let reparsed = Target::from_str(&formatted).unwrap();
        prop_assert_eq!(target, reparsed);
    }

    #[test]
    fn action_round_trips(s in valid_action_string()) {
        let action = Action::from_str(&s).unwrap();
        let formatted = action.to_string();
        let reparsed = Action::from_str(&formatted).unwrap();
        prop_assert_eq!(action, reparsed);
    }

    #[test]
    fn means_round_trips(s in valid_means_string()) {
        let means = Means::from_str(&s).unwrap();
        let formatted = means.to_string();
        let reparsed = Means::from_str(&formatted).unwrap();
        prop_assert_eq!(means, reparsed);
    }

    #[test]
    fn ichi_code_round_trips(s in valid_ichi_code_string()) {
        let code = IchiCode::from_str(&s).unwrap();
        let formatted = code.to_string();
        let reparsed = IchiCode::from_str(&formatted).unwrap();
        prop_assert_eq!(code, reparsed);
    }

    #[test]
    fn ichi_code_lowercase_round_trips(s in valid_ichi_code_string()) {
        let lower = s.to_ascii_lowercase();
        let code = IchiCode::from_str(&lower).unwrap();
        prop_assert_eq!(code.to_string(), s);
    }

    #[test]
    fn target_parser_never_panics(s in ".*") {
        let _ = Target::from_str(&s);
    }

    #[test]
    fn action_parser_never_panics(s in ".*") {
        let _ = Action::from_str(&s);
    }

    #[test]
    fn means_parser_never_panics(s in ".*") {
        let _ = Means::from_str(&s);
    }

    #[test]
    fn ichi_code_parser_never_panics(s in ".*") {
        let _ = IchiCode::from_str(&s);
    }
}
