//! `serde` round-trip tests. The whole file is a no-op when the `serde`
//! feature is disabled.
#![cfg(feature = "serde")]

use who_fic_linearization::LinearizationReader;

const MMS_HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tPrimary tabulation\tGrouping1\tGrouping2\tGrouping3\tGrouping4\tGrouping5\tVersion:2026 Apr 15 - 14:33 UTC\n";

const MMS_CATEGORY_ROW: &str = "http://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-m/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F257068234\"\",\"\"browser\"\")\"\tTrue\t0\tTrue\tBlockL1-1A0\tBlockL2-1A0\t\t\t\n";

const ICF_HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC\n";

const ICF_RESIDUAL_ROW: &str = "\thttp://id.who.int/icd/release/11/beta/icf/1060289548/other\tb1108\t\t\"- - - - - Other specified consciousness functions\"\tcategory\t2\tTrue\tTrue\t\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-icf/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F1060289548%2Ficf%2Fother\"\",\"\"browser\"\")\"\tTrue\t0\n";

#[test]
fn mms_row_round_trips_through_json() {
    let input = format!("{MMS_HEADER}{MMS_CATEGORY_ROW}");
    let row = LinearizationReader::from_str(&input)
        .next()
        .unwrap()
        .unwrap();

    let json = serde_json::to_string(&row).unwrap();
    let back = serde_json::from_str(&json).unwrap();
    assert_eq!(row, back);
}

#[test]
fn residual_row_round_trips_through_json() {
    let input = format!("{ICF_HEADER}{ICF_RESIDUAL_ROW}");
    let row = LinearizationReader::from_str(&input)
        .next()
        .unwrap()
        .unwrap();

    let json = serde_json::to_string(&row).unwrap();
    let back = serde_json::from_str(&json).unwrap();
    assert_eq!(row, back);
}

#[test]
fn serialized_row_contains_expected_fields() {
    let input = format!("{MMS_HEADER}{MMS_CATEGORY_ROW}");
    let row = LinearizationReader::from_str(&input)
        .next()
        .unwrap()
        .unwrap();

    let value: serde_json::Value = serde_json::to_value(&row).unwrap();
    assert_eq!(value["code"], serde_json::json!("1A00"));
    assert_eq!(value["title"], serde_json::json!("Cholera"));
    assert_eq!(
        value["groupings"],
        serde_json::json!(["BlockL1-1A0", "BlockL2-1A0"])
    );
}
