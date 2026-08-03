use std::str::FromStr;
use who_fic_icd::icd10::Icd10Code;
use who_fic_icd::icd11::Icd11Code;

fn main() {
    let code = Icd10Code::from_str("I63.9").unwrap();
    assert_eq!(code.category(), "I63");
    assert_eq!(code.subdivision(), Some("9"));

    let code = Icd11Code::from_str("8B20").unwrap();
    assert!(code.chapter().is_some());

    println!("ok");
}
