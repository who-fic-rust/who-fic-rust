//! Property tests: parse -> format -> parse round trips, and panic-safety
//! of the parsers on arbitrary input.

use proptest::prelude::*;
use std::str::FromStr;
use who_fic_icf::{IcfCode, QualifiedIcfCode};

/// A valid component letter.
fn component_letter() -> impl Strategy<Value = char> {
    prop_oneof![Just('b'), Just('s'), Just('d'), Just('e')]
}

/// A valid digit count for an `IcfCode`.
fn digit_count() -> impl Strategy<Value = usize> {
    prop_oneof![Just(1usize), Just(3), Just(4), Just(5)]
}

fn valid_icf_code_string() -> impl Strategy<Value = String> {
    (component_letter(), digit_count()).prop_flat_map(|(c, n)| {
        proptest::collection::vec(
            proptest::sample::select(vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
            n,
        )
        .prop_map(move |digits| {
            let mut s = String::new();
            s.push(c);
            s.extend(digits);
            s
        })
    })
}

fn valid_qualifier_digit() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('0'),
        Just('1'),
        Just('2'),
        Just('3'),
        Just('4'),
        Just('8'),
        Just('9'),
    ]
}

/// A valid `QualifiedIcfCode` string, built per-component so the qualifier
/// arity and separator always match the component.
fn valid_qualified_icf_code_string() -> impl Strategy<Value = String> {
    prop_oneof![
        // b: 1 qualifier digit, '.'
        (
            proptest::collection::vec(
                proptest::sample::select(vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
                1
            ),
            valid_qualifier_digit()
        )
            .prop_map(|(digits, q)| format!(
                "b{}.{}",
                digits.iter().collect::<String>(),
                q
            )),
        // s: 1-3 qualifier digits, '.'
        (1..=3usize).prop_flat_map(|n| {
            proptest::collection::vec(valid_qualifier_digit(), n)
                .prop_map(move |qs| format!("s730.{}", qs.iter().collect::<String>()))
        }),
        // d: 1-4 qualifier digits, '.'
        (1..=4usize).prop_flat_map(|n| {
            proptest::collection::vec(valid_qualifier_digit(), n)
                .prop_map(move |qs| format!("d450.{}", qs.iter().collect::<String>()))
        }),
        // e: 1 qualifier digit, '.' or '+'
        (valid_qualifier_digit(), prop_oneof![Just('.'), Just('+')])
            .prop_map(|(q, sep)| format!("e150{sep}{q}")),
    ]
}

proptest! {
    #[test]
    fn icf_code_round_trips(s in valid_icf_code_string()) {
        let code = IcfCode::from_str(&s).unwrap();
        let formatted = code.to_string();
        let reparsed = IcfCode::from_str(&formatted).unwrap();
        prop_assert_eq!(code, reparsed);
    }

    #[test]
    fn qualified_icf_code_round_trips(s in valid_qualified_icf_code_string()) {
        let code = QualifiedIcfCode::from_str(&s).unwrap();
        let formatted = code.to_string();
        let reparsed = QualifiedIcfCode::from_str(&formatted).unwrap();
        prop_assert_eq!(code, reparsed);
    }

    #[test]
    fn icf_code_parser_never_panics(s in ".*") {
        let _ = IcfCode::from_str(&s);
    }

    #[test]
    fn qualified_icf_code_parser_never_panics(s in ".*") {
        let _ = QualifiedIcfCode::from_str(&s);
    }

    #[test]
    fn parent_is_monotone_in_level(s in valid_icf_code_string()) {
        let code = IcfCode::from_str(&s).unwrap();
        let digit_count = code.as_str().len() - 1;
        match code.parent() {
            Some(parent) => {
                let parent_digit_count = parent.as_str().len() - 1;
                prop_assert!(parent_digit_count < digit_count);
            }
            None => {
                // Only chapters (1 digit) have no parent.
                prop_assert_eq!(digit_count, 1);
            }
        }
    }
}
