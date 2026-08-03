use std::str::FromStr;
use who_fic_icf::{Component, IcfCode, QualifiedIcfCode};

fn main() {
    let code = IcfCode::from_str("b280").unwrap();
    assert_eq!(code.component(), Component::BodyFunctions);
    assert_eq!(code.parent().unwrap().as_str(), "b2");

    let qualified = QualifiedIcfCode::from_str("b280.2").unwrap();
    assert_eq!(qualified.code(), &code);

    assert!(QualifiedIcfCode::from_str("e150.2").is_ok());
    assert!(QualifiedIcfCode::from_str("e150+2").is_ok());
    assert!(QualifiedIcfCode::from_str("b280+2").is_err());

    println!("ok");
}
