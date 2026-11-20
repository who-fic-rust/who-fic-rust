//! Property tests: iterating a [`LinearizationReader`] never panics, no
//! matter what bytes it is fed — from arbitrary garbage to inputs shaped
//! like a real export with random perturbations.

use proptest::prelude::*;
use who_fic_linearization::LinearizationReader;

/// One column of a plausible-looking row: sometimes bare, sometimes
/// quoted, sometimes deliberately broken quoting.
fn column_value() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z0-9/:._-]{0,12}",
        "\"[a-zA-Z0-9 -]{0,12}\"",
        Just("True".to_string()),
        Just("False".to_string()),
        Just(String::new()),
    ]
}

/// A line shaped like a plausible data row: a random number of
/// tab-joined columns.
fn row_like_line() -> impl Strategy<Value = String> {
    proptest::collection::vec(column_value(), 0..20).prop_map(|columns| columns.join("\t"))
}

/// A whole plausible-shaped file: a header-like line followed by some
/// number of row-like lines.
fn file_like_input() -> impl Strategy<Value = String> {
    proptest::collection::vec(row_like_line(), 0..8).prop_map(|lines| lines.join("\n"))
}

proptest! {
    #[test]
    fn iterating_arbitrary_bytes_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let reader = LinearizationReader::from_reader(std::io::Cursor::new(bytes));
        for result in reader {
            let _ = result;
        }
    }

    #[test]
    fn iterating_arbitrary_strings_never_panics(s in ".{0,2048}") {
        let reader = LinearizationReader::from_str(&s);
        for result in reader {
            let _ = result;
        }
    }

    #[test]
    fn iterating_row_shaped_garbage_never_panics(input in file_like_input()) {
        let reader = LinearizationReader::from_str(&input);
        for result in reader {
            let _ = result;
        }
    }

    #[test]
    fn line_numbers_never_exceed_the_number_of_lines(input in file_like_input()) {
        let total_lines = input.lines().count().max(1);
        let reader = LinearizationReader::from_str(&input);
        for result in reader {
            if let Err(error) = result {
                prop_assert!(error.line() >= 1);
                prop_assert!(error.line() <= total_lines);
            }
        }
    }
}
