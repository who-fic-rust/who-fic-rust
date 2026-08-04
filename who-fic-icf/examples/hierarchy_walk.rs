//! ICF codes form an explicit chapter -> second -> third -> fourth level
//! tree, derivable from the code's own digit count alone -- no data file
//! needed. This walks a fourth-level code all the way up to its chapter,
//! printing each ancestor and its level.
//!
//! Run with: `cargo run --example hierarchy_walk -p who-fic-icf`

use std::str::FromStr;
use who_fic_icf::IcfCode;

fn main() {
    let mut current = Some(IcfCode::from_str("b28010").unwrap());

    while let Some(code) = current {
        println!(
            "{:<8} level={:?} component={:?}",
            code.as_str(),
            code.level(),
            code.component()
        );
        current = code.parent();
    }

    // `chapter()` jumps straight to the top without walking one level at a
    // time, and every level is its own descendant/ancestor of the chapter.
    let leaf = IcfCode::from_str("b28010").unwrap();
    let chapter = leaf.chapter();
    assert_eq!(chapter.as_str(), "b2");
    assert!(chapter.is_ancestor_of(&leaf));
    assert!(leaf.is_descendant_of(&chapter));
    assert!(!leaf.is_ancestor_of(&chapter)); // not symmetric

    println!("\nok");
}
