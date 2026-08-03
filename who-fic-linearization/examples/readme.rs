use who_fic_linearization::LinearizationReader;

fn main() {
    let tsv = "\u{feff}Foundation URI\tLinearization (release) URI\tCode\tBlockId\tTitle\tClassKind\tDepthInKind\tIsResidual\tPrimaryLocation\tChapterNo\tBrowserLink\tisLeaf\tnoOfNonResidualChildren\tVersion:x\n\
               http://id.who.int/icd/entity/257068234\thttp://id.who.int/icd/release/11/beta/mms/257068234\t1A00\t\t\"- - - Cholera\"\tcategory\t1\tFalse\tTrue\t01\t\"link\"\tTrue\t0\n";

    for result in LinearizationReader::from_str(tsv) {
        let row = result.unwrap();
        assert_eq!(row.code(), Some("1A00"));
        assert_eq!(row.title(), "Cholera");
    }

    println!("ok");
}
