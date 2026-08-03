//! `serde` round-trip tests. The whole file is a no-op when the `serde`
//! feature is disabled.
#![cfg(feature = "serde")]

use std::str::FromStr;
use who_fic_icf::{Component, IcfCode, Level, QualifiedIcfCode, Qualifier};

#[test]
fn icf_code_serializes_as_canonical_string() {
    let code = IcfCode::from_str("b280").unwrap();
    let json = serde_json::to_string(&code).unwrap();
    assert_eq!(json, "\"b280\"");
}

#[test]
fn icf_code_round_trips_through_json() {
    for s in ["b2", "b280", "b2801", "b28010", "s730", "e150"] {
        let code = IcfCode::from_str(s).unwrap();
        let json = serde_json::to_string(&code).unwrap();
        let back: IcfCode = serde_json::from_str(&json).unwrap();
        assert_eq!(code, back);
    }
}

#[test]
fn icf_code_deserialize_rejects_invalid() {
    let result: Result<IcfCode, _> = serde_json::from_str("\"x280\"");
    assert!(result.is_err());
}

#[test]
fn qualified_icf_code_serializes_as_canonical_string() {
    let code = QualifiedIcfCode::from_str("b280.2").unwrap();
    let json = serde_json::to_string(&code).unwrap();
    assert_eq!(json, "\"b280.2\"");
}

#[test]
fn qualified_icf_code_round_trips_through_json() {
    for s in ["b280.2", "s730.312", "d450.12", "e150+2", "e150.2"] {
        let code = QualifiedIcfCode::from_str(s).unwrap();
        let json = serde_json::to_string(&code).unwrap();
        let back: QualifiedIcfCode = serde_json::from_str(&json).unwrap();
        assert_eq!(code, back);
    }
}

#[test]
fn qualified_icf_code_deserialize_rejects_invalid() {
    let result: Result<QualifiedIcfCode, _> = serde_json::from_str("\"b280+2\"");
    assert!(result.is_err());
}

#[test]
fn component_qualifier_level_round_trip_through_json() {
    let component = Component::EnvironmentalFactors;
    let json = serde_json::to_string(&component).unwrap();
    let back: Component = serde_json::from_str(&json).unwrap();
    assert_eq!(component, back);

    let qualifier = Qualifier::Moderate;
    let json = serde_json::to_string(&qualifier).unwrap();
    let back: Qualifier = serde_json::from_str(&json).unwrap();
    assert_eq!(qualifier, back);

    let level = Level::ThirdLevel;
    let json = serde_json::to_string(&level).unwrap();
    let back: Level = serde_json::from_str(&json).unwrap();
    assert_eq!(level, back);
}
