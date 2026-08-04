//! ICD-11 postcoordination: combining a stem code with extension codes (or
//! multiple stem codes) into one cluster describing a more specific
//! clinical concept than any single code alone.
//!
//! This is syntax-only, as `who-fic-icd` always is: parsing and formatting
//! a cluster string, not validating which extensions WHO actually permits
//! on a given stem (that requires WHO's data — see `specs/who-fic-icd.md`).
//!
//! Run with: `cargo run --example postcoordination_cluster -p who-fic-icd`

use std::str::FromStr;
use who_fic_icd::icd11::Cluster;

fn main() {
    // '&' attaches one or more extension codes to a stem code.
    let cluster = Cluster::from_str("8B20&XK9J").unwrap();
    println!("Parsed: {cluster}");
    for group in cluster.groups() {
        println!("  stem: {}", group.stem());
        for extension in group.extensions() {
            println!("    + extension: {extension}");
        }
    }
    assert_eq!(cluster.stems().count(), 1);
    assert_eq!(cluster.extensions().count(), 1);

    println!();

    // '/' joins multiple stem codes into one clinical concept.
    let multi_stem = Cluster::from_str("NA07.1/8B20").unwrap();
    println!("Parsed: {multi_stem}");
    for stem in multi_stem.stems() {
        println!("  stem: {stem}");
    }
    assert_eq!(multi_stem.stems().count(), 2);
    assert_eq!(multi_stem.extensions().count(), 0);

    // Round trip: formatting a parsed cluster reproduces its canonical form.
    assert_eq!(cluster.to_string(), "8B20&XK9J");
    assert_eq!(multi_stem.to_string(), "NA07.1/8B20");

    println!("\nok");
}
