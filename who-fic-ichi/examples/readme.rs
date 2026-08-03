use who_fic_ichi::IchiCode;

fn main() {
    let code: IchiCode = "kab.db.ad".parse().unwrap();
    assert_eq!(code.to_string(), "KAB.DB.AD");
    assert_eq!(code.target().as_str(), "KAB");
    assert_eq!(code.action().as_str(), "DB");
    assert_eq!(code.means().as_str(), "AD");

    println!("ok");
}
