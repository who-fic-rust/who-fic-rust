//! ClaML expresses hierarchy by reference (`SuperClass`/`SubClass` code
//! attributes), not by XML nesting -- so walking "up" from a class to its
//! ancestors means looking its parent's code up in the flat `classes()`
//! list, not walking the DOM. This builds a tiny code -> Class lookup and
//! walks a category up to its chapter.
//!
//! The XML below is a small hand-written fixture in the real ClaML shape
//! (see `specs/who-fic-claml.md`), not a vendored WHO ICD-10 export.
//!
//! Run with: `cargo run --example walk_hierarchy -p who-fic-claml`

use std::collections::HashMap;
use std::str::FromStr;
use who_fic_claml::{ClamlDocument, Class};

const XML: &str = r#"
<ClaML version="2.0">
  <Class code="I" kind="chapter">
    <Rubric kind="preferred"><Label xml:lang="en">Certain infectious or parasitic diseases</Label></Rubric>
  </Class>
  <Class code="A00-A09" kind="block">
    <SuperClass code="I"/>
    <Rubric kind="preferred"><Label xml:lang="en">Intestinal infectious diseases</Label></Rubric>
  </Class>
  <Class code="A00" kind="category">
    <SuperClass code="A00-A09"/>
    <Rubric kind="preferred"><Label xml:lang="en">Cholera</Label></Rubric>
  </Class>
</ClaML>
"#;

fn main() {
    let doc = ClamlDocument::from_str(XML).unwrap();

    // A simple code -> Class index, since ClaML itself is just a flat list.
    let by_code: HashMap<&str, &Class> = doc.classes().iter().map(|c| (c.code(), c)).collect();

    let mut current = by_code.get("A00").copied();
    while let Some(class) = current {
        println!(
            "{:<8} [{}] {}",
            class.code(),
            class.kind(),
            class.preferred_label("en").unwrap_or("(no title)")
        );
        // A Class can have more than one SuperClass in principle; this
        // walks the first, which covers the common single-parent case.
        current = class
            .super_classes()
            .first()
            .and_then(|parent_code| by_code.get(parent_code.as_str()))
            .copied();
    }

    println!("\nok");
}
