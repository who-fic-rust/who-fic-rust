//! Every ICHI code is three independently-meaningful axes -- Target,
//! Action, Means -- composed as `TARGET.ACTION.MEANS`. This shows building
//! a code both ways (parsing the dotted form, and composing typed axis
//! values directly) and pulling the axes back apart.
//!
//! Run with: `cargo run --example axis_composition -p who-fic-ichi`

use std::str::FromStr;
use who_fic_ichi::{Action, IchiCode, Means, Target};

fn main() {
    // Parse the dotted form.
    let parsed: IchiCode = "kab.db.ad".parse().unwrap(); // case-insensitive
    assert_eq!(parsed.to_string(), "KAB.DB.AD"); // canonical uppercase

    // Or compose it from independently-validated axis values -- each axis
    // type is meaningful on its own (WHO publishes them as standalone
    // value sets), not just a private field of IchiCode.
    let target = Target::from_str("KAB").unwrap();
    let action = Action::from_str("DB").unwrap();
    let means = Means::from_str("AD").unwrap();
    let composed = IchiCode::from_parts(target, action, means);

    assert_eq!(parsed, composed);

    // Pull the axes back apart from either one.
    for code in [&parsed, &composed] {
        println!(
            "{code}  target={} action={} means={}",
            code.target(),
            code.action(),
            code.means()
        );
    }

    // A malformed axis fails independently of the others -- useful when
    // reporting which specific segment of a dotted code was wrong.
    let err = IchiCode::from_str("KAB.D.AD").unwrap_err(); // action too short
    println!("\nexpected error: {err}");

    println!("\nok");
}
