//! Streaming a multi-row export and filtering by row kind -- the shape
//! `who-fic-icd`/`who-fic-icf`/`who-fic-ichi`'s own data-loading `*Index`
//! types build on top of. This crate has no opinion on classification
//! codes, so this example just demonstrates the row stream itself: walking
//! every row, telling chapter/block/category rows apart, and handling a
//! per-row parse error without aborting the whole stream where possible.
//!
//! The TSV below is a small hand-written fixture in the real export shape
//! (see `specs/who-fic-linearization.md`), not a vendored WHO file.
//!
//! Run with: `cargo run --example stream_and_filter -p who-fic-linearization`

use who_fic_linearization::LinearizationReader;

const TSV: &str = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
http://id.who.int/icd/entity/1\thttp://id.who.int/icd/release/11/beta/mms/1\t\t\t\"Certain infectious or parasitic diseases\"\tchapter\t1\tFalse\tTrue\t01\t\"link\"\tFalse\t2\n\
http://id.who.int/icd/entity/2\thttp://id.who.int/icd/release/11/beta/mms/2\t\tBlockL1-1A0\t\"- Gastroenteritis or colitis of infectious origin\"\tblock\t1\tFalse\tTrue\t01\t\"link\"\tFalse\t1\n\
http://id.who.int/icd/entity/3\thttp://id.who.int/icd/release/11/beta/mms/3\t1A00\t\t\"- - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"link\"\tTrue\t0\n\
\thttp://id.who.int/icd/release/11/beta/mms/3/other\t1A00.Y\t\t\"- - - Cholera, other specified\"\tcategory\t2\tTrue\tTrue\t01\t\"link\"\tTrue\t0\n";

fn main() {
    let reader = LinearizationReader::from_str(TSV);

    let mut categories = 0;
    let mut residuals = 0;
    let mut non_leaf = 0;

    for result in reader {
        let row = match result {
            Ok(row) => row,
            Err(err) => {
                // A malformed *row* doesn't mean the rest of the file is
                // unreadable -- report and move on, same policy the
                // classification crates' *Index adapters use for a code
                // that fails to parse (as opposed to the reader itself
                // failing to open/decode the file, which is unrecoverable).
                eprintln!("skipping malformed row: {err}");
                continue;
            }
        };

        match row.class_kind() {
            "chapter" | "block" => non_leaf += 1,
            "category" => {
                categories += 1;
                if row.is_residual() {
                    residuals += 1;
                }
                println!(
                    "{:<8} {}{}",
                    row.code().unwrap_or("?"),
                    row.title(),
                    if row.is_residual() {
                        "  [residual]"
                    } else {
                        ""
                    }
                );
            }
            other => println!("(unrecognized ClassKind {other:?}, skipping)"),
        }
    }

    println!("\n{non_leaf} chapter/block rows, {categories} categories ({residuals} residual)");
    assert_eq!((non_leaf, categories, residuals), (2, 2, 1));
    println!("ok");
}
