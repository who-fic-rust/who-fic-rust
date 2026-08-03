//! `serde` round-trip tests. The whole file is a no-op when the `serde`
//! feature is disabled.
#![cfg(feature = "serde")]

use std::str::FromStr;
use who_fic_ichi::{Action, Axis, IchiCode, Means, Section, Target};

#[test]
fn target_serializes_as_canonical_string() {
    let target = Target::from_str("kab").unwrap();
    let json = serde_json::to_string(&target).unwrap();
    assert_eq!(json, "\"KAB\"");
}

#[test]
fn action_serializes_as_canonical_string() {
    let action = Action::from_str("db").unwrap();
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"DB\"");
}

#[test]
fn means_serializes_as_canonical_string() {
    let means = Means::from_str("ad").unwrap();
    let json = serde_json::to_string(&means).unwrap();
    assert_eq!(json, "\"AD\"");
}

#[test]
fn ichi_code_serializes_as_canonical_string() {
    let code = IchiCode::from_str("kab.db.ad").unwrap();
    let json = serde_json::to_string(&code).unwrap();
    assert_eq!(json, "\"KAB.DB.AD\"");
}

#[test]
fn ichi_code_round_trips_through_json() {
    for s in ["KAB.DB.AD", "AAA.FA.AE", "000.00.00"] {
        let code = IchiCode::from_str(s).unwrap();
        let json = serde_json::to_string(&code).unwrap();
        let back: IchiCode = serde_json::from_str(&json).unwrap();
        assert_eq!(code, back);
    }
}

#[test]
fn target_action_means_round_trip_through_json() {
    let target = Target::from_str("KAB").unwrap();
    let json = serde_json::to_string(&target).unwrap();
    let back: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(target, back);

    let action = Action::from_str("DB").unwrap();
    let json = serde_json::to_string(&action).unwrap();
    let back: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(action, back);

    let means = Means::from_str("AD").unwrap();
    let json = serde_json::to_string(&means).unwrap();
    let back: Means = serde_json::from_str(&json).unwrap();
    assert_eq!(means, back);
}

#[test]
fn ichi_code_deserialize_rejects_invalid() {
    let result: Result<IchiCode, _> = serde_json::from_str("\"KA.DB.AD\"");
    assert!(result.is_err());
}

#[test]
fn target_deserialize_rejects_invalid() {
    let result: Result<Target, _> = serde_json::from_str("\"K@B\"");
    assert!(result.is_err());
}

#[test]
fn section_and_axis_round_trip_through_json() {
    let section = Section::Environment;
    let json = serde_json::to_string(&section).unwrap();
    let back: Section = serde_json::from_str(&json).unwrap();
    assert_eq!(section, back);

    let axis = Axis::Target;
    let json = serde_json::to_string(&axis).unwrap();
    let back: Axis = serde_json::from_str(&json).unwrap();
    assert_eq!(axis, back);
}
