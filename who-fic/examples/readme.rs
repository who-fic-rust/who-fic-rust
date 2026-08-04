#[cfg(all(feature = "icd", feature = "icf", feature = "ichi"))]
fn run() {
    use std::str::FromStr;

    let code = who_fic::icd::icd11::Icd11Code::from_str("8B20").unwrap();
    assert_eq!(code.as_str(), "8B20");

    let code = who_fic::icf::IcfCode::from_str("b280").unwrap();
    assert_eq!(code.component(), who_fic::icf::Component::BodyFunctions);

    let code = who_fic::ichi::IchiCode::from_str("KAB.DB.AD").unwrap();
    assert_eq!(code.target().as_str(), "KAB");

    println!("ok");
}

#[cfg(not(all(feature = "icd", feature = "icf", feature = "ichi")))]
fn run() {}

fn main() {
    run();
}
