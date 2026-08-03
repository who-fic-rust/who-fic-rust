//! Fixture-based tests against small, hand-written excerpts that reproduce
//! the *exact* shape of real WHO Simplified Linearization Output exports
//! (verified against real downloads, 2026-08 — see
//! `specs/who-fic-linearization.md`). These are not vendored WHO files:
//! each fixture below is a handful of representative lines, not a full
//! export.

use who_fic_linearization::{LinearizationError, LinearizationReader};

const ICF_HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:2026 Apr 15 - 14:33 UTC";

const MMS_HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tPrimary tabulation\tGrouping1\tGrouping2\tGrouping3\tGrouping4\tGrouping5\tVersion:2026 Apr 15 - 14:33 UTC";

/// Real ICF chapter row (`ClassKind` = `chapter`), from
/// `LinearizationMiniOutput-ICF-en.txt`.
const ICF_CHAPTER_ROW: &str = "http://id.who.int/icd/entity/619527855\thttp://id.who.int/icd/release/11/beta/icf/619527855\t\t\t\"ICF Category\"\tchapter\t1\tFalse\tTrue\t\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-icf/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F619527855\"\",\"\"browser\"\")\"\tFalse\t4";

/// Real ICF category row (`b110`), depth 1, four `"- "` markers.
const ICF_CATEGORY_ROW: &str = "http://id.who.int/icd/entity/1060289548\thttp://id.who.int/icd/release/11/beta/icf/1060289548\tb110\t\t\"- - - - Consciousness functions\"\tcategory\t1\tFalse\tTrue\t\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-icf/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F1060289548\"\",\"\"browser\"\")\"\tFalse\t3";

/// Real ICF residual row: empty `Foundation URI`, `/other` linearization
/// URI suffix.
const ICF_RESIDUAL_ROW: &str = "\thttp://id.who.int/icd/release/11/beta/icf/1060289548/other\tb1108\t\t\"- - - - - Other specified consciousness functions\"\tcategory\t2\tTrue\tTrue\t\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-icf/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F1060289548%2Ficf%2Fother\"\",\"\"browser\"\")\"\tTrue\t0";

/// Real MMS chapter row with all trailing MMS-only columns omitted (a
/// genuinely short line), from `LinearizationMiniOutput-MMS-en.txt`.
const MMS_CHAPTER_ROW_SHORT: &str = "http://id.who.int/icd/entity/1435254666\thttp://id.who.int/icd/release/11/beta/mms/1435254666\t\t\t\"Certain infectious or parasitic diseases\"\tchapter\t1\tFalse\tTrue\t01\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-m/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F1435254666\"\",\"\"browser\"\")\"\tFalse\t22";

/// Real MMS block row (`BlockId` = `BlockL2-1A0`), with `Grouping1` set.
const MMS_BLOCK_ROW: &str = "http://id.who.int/icd/entity/135352227\thttp://id.who.int/icd/release/11/beta/mms/135352227\t\tBlockL2-1A0\t\"- - Bacterial intestinal infections\"\tblock\t2\tFalse\tTrue\t01\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-m/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F135352227\"\",\"\"browser\"\")\"\tFalse\t8\t\tBlockL1-1A0\t\t\t\t";

/// Real MMS category row (`1A00`, Cholera) with two `Grouping` columns set.
const MMS_CATEGORY_ROW: &str = "http://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-m/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F257068234\"\",\"\"browser\"\")\"\tTrue\t0\tTrue\tBlockL1-1A0\tBlockL2-1A0\t\t\t";

/// Real MMS residual row (`1A03.Y`): empty `Foundation URI`, `/other`
/// linearization URI suffix.
const MMS_RESIDUAL_ROW: &str = "\thttp://id.who.int/icd/release/11/beta/mms/344162786/other\t1A03.Y\t\t\"- - - - Intestinal infections due to other specified Escherichia coli\"\tcategory\t2\tTrue\tTrue\t01\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-m/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F344162786%2Fmms%2Fother\"\",\"\"browser\"\")\"\tTrue\t0\tTrue\tBlockL1-1A0\tBlockL2-1A0\t\t\t";

/// Real ICHI row (`??.BA.BH`), from `LinearizationMiniOutput-ICHI-en.txt`
/// (13-column header, identical shape to ICF).
const ICHI_ROW: &str = "http://id.who.int/icd/entity/84065804\thttp://id.who.int/icd/release/11/beta/ichi/84065804\t??.BA.BH\t\t\"- - - - - Magnetic resonance angiography of intracranial vessels (proposed)\"\tcategory\t1\tFalse\tTrue\t\t\"=hyperlink(\"\"https://icd.who.int/dev11/l-ichi/en#/http%3A%2F%2Fid.who.int%2Ficd%2Fentity%2F84065804\"\",\"\"browser\"\")\"\tTrue\t0";

fn rows(header: &str, data_lines: &[&str]) -> Vec<who_fic_linearization::LinearizationRow> {
    let mut tsv = String::from(header);
    tsv.push('\n');
    for line in data_lines {
        tsv.push_str(line);
        tsv.push('\n');
    }
    LinearizationReader::from_str(&tsv)
        .collect::<Result<_, _>>()
        .expect("fixture rows should parse")
}

#[test]
fn chapter_row() {
    let parsed = rows(ICF_HEADER, &[ICF_CHAPTER_ROW]);
    let row = &parsed[0];
    assert_eq!(row.class_kind(), "chapter");
    assert_eq!(row.code(), None);
    assert_eq!(row.block_id(), None);
    assert_eq!(row.title(), "ICF Category");
    assert_eq!(row.depth_in_kind(), 1);
    assert!(!row.is_residual());
    assert_eq!(row.no_of_non_residual_children(), 4);
    assert_eq!(
        row.foundation_uri(),
        Some("http://id.who.int/icd/entity/619527855")
    );
}

#[test]
fn block_row() {
    let parsed = rows(MMS_HEADER, &[MMS_BLOCK_ROW]);
    let row = &parsed[0];
    assert_eq!(row.class_kind(), "block");
    assert_eq!(row.code(), None);
    assert_eq!(row.block_id(), Some("BlockL2-1A0"));
    assert_eq!(row.title(), "Bacterial intestinal infections");
    assert_eq!(row.depth_in_kind(), 2);
    assert_eq!(row.groupings(), &["BlockL1-1A0".to_string()]);
}

#[test]
fn category_row() {
    let parsed = rows(MMS_HEADER, &[MMS_CATEGORY_ROW]);
    let row = &parsed[0];
    assert_eq!(row.class_kind(), "category");
    assert_eq!(row.code(), Some("1A00"));
    assert_eq!(row.block_id(), None);
    assert_eq!(row.title(), "Cholera");
    assert!(!row.is_residual());
    assert!(row.is_leaf());
    assert_eq!(row.primary_tabulation(), Some(true));
    assert_eq!(
        row.groupings(),
        &["BlockL1-1A0".to_string(), "BlockL2-1A0".to_string()]
    );
}

#[test]
fn residual_row_has_empty_foundation_uri() {
    for (header, residual) in [
        (ICF_HEADER, ICF_RESIDUAL_ROW),
        (MMS_HEADER, MMS_RESIDUAL_ROW),
    ] {
        let parsed = rows(header, &[residual]);
        let row = &parsed[0];
        assert_eq!(row.foundation_uri(), None);
        assert!(row.is_residual());
        assert!(row.linearization_uri().ends_with("/other"));
    }
}

#[test]
fn mms_residual_row_code_and_groupings() {
    let parsed = rows(MMS_HEADER, &[MMS_RESIDUAL_ROW]);
    let row = &parsed[0];
    assert_eq!(row.code(), Some("1A03.Y"));
    assert_eq!(
        row.groupings(),
        &["BlockL1-1A0".to_string(), "BlockL2-1A0".to_string()]
    );
}

#[test]
fn short_line_missing_trailing_columns_defaults_to_none() {
    let parsed = rows(MMS_HEADER, &[MMS_CHAPTER_ROW_SHORT]);
    let row = &parsed[0];
    assert_eq!(row.class_kind(), "chapter");
    assert_eq!(row.no_of_non_residual_children(), 22);
    // The line ends right after `noOfNonResidualChildren`; every MMS-only
    // trailing column is missing outright, not present-but-empty.
    assert_eq!(row.primary_tabulation(), None);
    assert_eq!(row.groupings(), &[] as &[String]);
}

#[test]
fn mms_row_with_grouping_columns_present() {
    let parsed = rows(MMS_HEADER, &[MMS_CATEGORY_ROW]);
    assert_eq!(parsed[0].groupings().len(), 2);
}

#[test]
fn icf_and_ichi_rows_have_no_groupings() {
    let icf = rows(ICF_HEADER, &[ICF_CATEGORY_ROW]);
    assert_eq!(icf[0].groupings(), &[] as &[String]);
    assert_eq!(icf[0].primary_tabulation(), None);

    let ichi = rows(ICF_HEADER, &[ICHI_ROW]);
    let row = &ichi[0];
    assert_eq!(row.code(), Some("??.BA.BH"));
    assert_eq!(row.groupings(), &[] as &[String]);
}

#[test]
fn malformed_line_reports_error_with_line_number() {
    let mut tsv = String::from(ICF_HEADER);
    tsv.push('\n');
    tsv.push_str(ICF_CHAPTER_ROW);
    tsv.push('\n');
    // Broken quoting: an opening quote with no closing quote anywhere in
    // the rest of the line.
    tsv.push_str("http://id.who.int/icd/entity/9\thttp://id.who.int/icd/release/11/beta/icf/9\t\t\t\"broken title\tcategory\t1\tFalse\tTrue\t\t\tTrue\t0");
    tsv.push('\n');

    let mut reader = LinearizationReader::from_str(&tsv);
    // Row 1 (line 2 of the file) parses fine.
    assert!(reader.next().unwrap().is_ok());
    // Row 2 (line 3 of the file) is malformed.
    let err = reader.next().unwrap().unwrap_err();
    assert_eq!(err, LinearizationError::UnterminatedQuotedField { line: 3 });
    assert_eq!(err.line(), 3);
}

#[test]
fn bom_is_stripped_from_header() {
    let mut tsv = String::from('\u{feff}');
    tsv.push_str(ICF_HEADER);
    tsv.push('\n');
    tsv.push_str(ICF_CATEGORY_ROW);
    tsv.push('\n');

    let mut reader = LinearizationReader::from_str(&tsv);
    let row = reader.next().unwrap().unwrap();
    assert_eq!(row.code(), Some("b110"));
    // The BOM must not leak into the first column of the first data row.
    assert_eq!(
        row.foundation_uri(),
        Some("http://id.who.int/icd/entity/1060289548")
    );
}

#[test]
fn title_depth_markers_are_stripped() {
    let parsed = rows(MMS_HEADER, &[MMS_CATEGORY_ROW]);
    assert_eq!(parsed[0].title(), "Cholera");

    let block = rows(MMS_HEADER, &[MMS_BLOCK_ROW]);
    assert_eq!(block[0].title(), "Bacterial intestinal infections");

    // A title with no depth markers at all is left untouched.
    let chapter = rows(ICF_HEADER, &[ICF_CHAPTER_ROW]);
    assert_eq!(chapter[0].title(), "ICF Category");
}

#[test]
fn browser_link_is_exposed_raw() {
    let parsed = rows(ICF_HEADER, &[ICF_CATEGORY_ROW]);
    let link = parsed[0].browser_link().expect("browser link present");
    assert!(link.starts_with("=hyperlink("));
    assert!(link.contains("\"browser\""));
}
