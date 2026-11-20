//! Regression tests: the umbrella `serde` feature must reach the parser
//! crates via weak forwarding through the classification crates
//! (`who-fic-icd/serde = [..., "who-fic-linearization?/serde", ...]`), so
//! the parser types callers pair with the adapter indexes are
//! serializable when both features are on. This was a real gap once —
//! see specs/who-fic.md and tasks.md Phase 17.

#![cfg(all(feature = "serde", feature = "icd"))]

#[cfg(feature = "linearization")]
mod linearization_forwarding {
    use who_fic_linearization::LinearizationReader;

    const MMS_HEADER: &str = "Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tPrimary tabulation\tGrouping1\tGrouping2\tGrouping3\tGrouping4\tGrouping5\tVersion:2026 Apr 15 - 14:33 UTC\n";
    const MMS_CATEGORY_ROW: &str = "http://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\tTrue\t0\tTrue\tBlockL1-1A0\t\t\t\t\n";

    #[test]
    fn umbrella_serde_reaches_linearization_row() {
        let input = format!("{MMS_HEADER}{MMS_CATEGORY_ROW}");
        let row = LinearizationReader::from_str(&input)
            .next()
            .unwrap()
            .unwrap();

        // The adapter index composes through the umbrella features…
        let index = who_fic::icd::icd11::linearization::Icd11LinearizationIndex::from_rows(
            std::iter::once(Ok(row.clone())),
        )
        .unwrap();
        assert_eq!(index.len(), 1);

        // …and the row itself is serializable, which is exactly what the
        // forwarding gap used to break.
        let json = serde_json::to_string(&row).unwrap();
        let back: who_fic_linearization::LinearizationRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, row);
    }
}

#[cfg(feature = "claml")]
mod claml_forwarding {
    use std::str::FromStr;
    use who_fic_claml::ClamlDocument;

    #[test]
    fn umbrella_serde_reaches_claml_document() {
        let doc = ClamlDocument::from_str(
            r#"<ClaML version="2.0.0">
                 <Class code="A00" kind="category">
                   <Rubric kind="preferred">
                     <Label xml:lang="en">Cholera</Label>
                   </Rubric>
                 </Class>
               </ClaML>"#,
        )
        .unwrap();

        let index = who_fic::icd::icd10::claml::Icd10ClamlIndex::from_document(&doc).unwrap();
        assert_eq!(index.len(), 1);

        let json = serde_json::to_string(&doc).unwrap();
        let back: ClamlDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(back, doc);
    }
}
