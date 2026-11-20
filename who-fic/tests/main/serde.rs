#![cfg(all(feature = "serde", feature = "icd", feature = "icf", feature = "ichi"))]

use std::str::FromStr;

#[test]
fn icd11_round_trips_through_umbrella_crate() {
    let code = who_fic::icd::icd11::Icd11Code::from_str("8B20").unwrap();
    let json = serde_json::to_string(&code).unwrap();
    assert_eq!(json, "\"8B20\"");
    let back: who_fic::icd::icd11::Icd11Code = serde_json::from_str(&json).unwrap();
    assert_eq!(back, code);
}

#[test]
fn icf_round_trips_through_umbrella_crate() {
    let code = who_fic::icf::IcfCode::from_str("b280").unwrap();
    let json = serde_json::to_string(&code).unwrap();
    assert_eq!(json, "\"b280\"");
    let back: who_fic::icf::IcfCode = serde_json::from_str(&json).unwrap();
    assert_eq!(back, code);
}

#[test]
fn ichi_round_trips_through_umbrella_crate() {
    let code = who_fic::ichi::IchiCode::from_str("KAB.DB.AD").unwrap();
    let json = serde_json::to_string(&code).unwrap();
    assert_eq!(json, "\"KAB.DB.AD\"");
    let back: who_fic::ichi::IchiCode = serde_json::from_str(&json).unwrap();
    assert_eq!(back, code);
}
