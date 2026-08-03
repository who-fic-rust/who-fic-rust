use std::str::FromStr;
use who_fic_claml::ClamlDocument;

fn main() {
    let xml = r#"
<ClaML version="2.0">
  <Class code="A00" kind="category">
    <SuperClass code="A00-A09"/>
    <Rubric kind="preferred">
      <Label xml:lang="en">Cholera</Label>
    </Rubric>
  </Class>
</ClaML>
"#;

    let doc = ClamlDocument::from_str(xml).unwrap();
    let class = &doc.classes()[0];
    assert_eq!(class.code(), "A00");
    assert_eq!(class.preferred_label("en"), Some("Cholera"));

    println!("ok");
}
