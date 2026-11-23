//! Internal event-based ClaML parser, built on `quick-xml`'s pull-parser
//! (`quick_xml::reader::Reader`) rather than any serde-derive style
//! deserialization. Not part of the public API.

use quick_xml::XmlVersion;
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::reader::Reader;

use crate::{ClamlDocument, ClamlError, Class, Label, Modifier, ModifierClass, Rubric};

/// Parses a complete ClaML document from a string already in memory.
pub(crate) fn parse_document(xml: &str) -> Result<ClamlDocument, ClamlError> {
    let mut reader = Reader::from_str(xml);
    loop {
        match read_event(&mut reader)? {
            Event::Eof => return Ok(ClamlDocument::default()),
            // Whatever the root element is named (normally `ClaML`), parse
            // its children generically; the root's own attributes (e.g.
            // `version`) are not modeled by this crate.
            Event::Start(_) => return parse_root_children(&mut reader),
            Event::Empty(_) => return Ok(ClamlDocument::default()),
            _ => {} // Decl, Comment, PI, DocType: skip while looking for the root element.
        }
    }
}

/// Reads the next event, mapping any `quick-xml` error into a [`ClamlError`]
/// with the position of the failure.
fn read_event<'a>(reader: &mut Reader<&'a [u8]>) -> Result<Event<'a>, ClamlError> {
    reader
        .read_event()
        .map_err(|source| ClamlError::xml(source, reader.error_position()))
}

/// Looks up a named attribute on a start/empty tag, decoding and
/// unescaping its value.
fn attr_value(
    element: &BytesStart<'_>,
    key: &str,
    reader: &Reader<&[u8]>,
) -> Result<Option<String>, ClamlError> {
    for result in element.attributes() {
        let attribute =
            result.map_err(|source| ClamlError::xml(source.into(), reader.buffer_position()))?;
        if attribute.key.as_ref() == key.as_bytes() {
            let value = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                .map_err(|source| ClamlError::xml(source, reader.buffer_position()))?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

/// Appends the resolved text of a `GeneralRef` event (`&name;` or
/// `&#NNN;`) to `text`. Unknown named entities (not one of the five
/// predefined XML entities) are passed through verbatim as `&name;`,
/// since resolving custom DTD-declared entities is out of scope.
fn append_general_ref(
    text: &mut String,
    reference: &BytesRef<'_>,
    reader: &Reader<&[u8]>,
) -> Result<(), ClamlError> {
    if let Some(ch) = reference
        .resolve_char_ref()
        .map_err(|source| ClamlError::xml(source, reader.buffer_position()))?
    {
        text.push(ch);
        return Ok(());
    }
    let name = reference
        .decode()
        .map_err(|source| ClamlError::xml(source.into(), reader.buffer_position()))?;
    match quick_xml::escape::resolve_predefined_entity(&name) {
        Some(resolved) => text.push_str(resolved),
        None => {
            text.push('&');
            text.push_str(&name);
            text.push(';');
        }
    }
    Ok(())
}

/// Reads events until the end tag matching the element whose start tag was
/// just consumed, discarding everything. Used for elements this crate does
/// not model (e.g. `Identifier`, `ClassKinds`).
fn skip_element(reader: &mut Reader<&[u8]>) -> Result<(), ClamlError> {
    let mut depth = 0i32;
    loop {
        match read_event(reader)? {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                if depth == 0 {
                    return Ok(());
                }
                depth -= 1;
            }
            Event::Eof => {
                return Err(ClamlError::unexpected_eof(
                    "element",
                    reader.buffer_position(),
                ));
            }
            _ => {}
        }
    }
}

/// Reads events until the end tag matching the element whose start tag was
/// just consumed, concatenating all text found anywhere within (including
/// inside nested markup, which is otherwise discarded) into a single
/// string. Used for `Title` and `Label`.
fn read_flattened_text(
    reader: &mut Reader<&[u8]>,
    element: &'static str,
) -> Result<String, ClamlError> {
    let mut text = String::new();
    let mut depth = 0i32;
    loop {
        match read_event(reader)? {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            Event::Text(t) => {
                let decoded = t
                    .decode()
                    .map_err(|source| ClamlError::xml(source.into(), reader.buffer_position()))?;
                text.push_str(&decoded);
            }
            Event::CData(t) => {
                let decoded = t
                    .decode()
                    .map_err(|source| ClamlError::xml(source.into(), reader.buffer_position()))?;
                text.push_str(&decoded);
            }
            Event::GeneralRef(r) => append_general_ref(&mut text, &r, reader)?,
            Event::Eof => {
                return Err(ClamlError::unexpected_eof(
                    element,
                    reader.buffer_position(),
                ));
            }
            _ => {}
        }
    }
    Ok(text.trim().to_string())
}

/// Parses the children of the root element: `Title`, `Class`,
/// `ModifierClass`, and anything else (skipped generically), up to the
/// root's own end tag.
fn parse_root_children(reader: &mut Reader<&[u8]>) -> Result<ClamlDocument, ClamlError> {
    let mut title = None;
    let mut classes = Vec::new();
    let mut modifier_classes = Vec::new();

    loop {
        match read_event(reader)? {
            Event::End(_) => break,
            Event::Eof => {
                return Err(ClamlError::unexpected_eof(
                    "ClaML",
                    reader.buffer_position(),
                ));
            }
            Event::Start(e) => match e.local_name().as_ref() {
                b"Title" => title = Some(read_flattened_text(reader, "Title")?),
                b"Class" => classes.push(parse_class(reader, &e)?),
                b"ModifierClass" => modifier_classes.push(parse_modifier_class(reader, &e)?),
                _ => skip_element(reader)?,
            },
            Event::Empty(e) => match e.local_name().as_ref() {
                b"Class" => classes.push(finish_class(reader, &e)?),
                b"ModifierClass" => modifier_classes.push(finish_modifier_class(reader, &e)?),
                _ => {}
            },
            _ => {}
        }
    }

    Ok(ClamlDocument {
        title,
        classes,
        modifier_classes,
    })
}

/// Builds a `Class` with no children, from a self-closing `<Class .../>`
/// tag.
fn finish_class(reader: &Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Class, ClamlError> {
    let code = attr_value(start, "code", reader)?
        .ok_or_else(|| ClamlError::missing_attribute("Class", "code", reader.buffer_position()))?;
    // RCA-IN-3: absent only; documented on `Class::kind` / `Label::lang` (absent attribute).
    let kind = attr_value(start, "kind", reader)?.unwrap_or_default();
    Ok(Class {
        code,
        kind,
        super_classes: Vec::new(),
        sub_classes: Vec::new(),
        rubrics: Vec::new(),
    })
}

/// Parses a `Class` element and its children, up to its own end tag.
fn parse_class(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Class, ClamlError> {
    let code = attr_value(start, "code", reader)?
        .ok_or_else(|| ClamlError::missing_attribute("Class", "code", reader.buffer_position()))?;
    // RCA-IN-3: absent only; documented on `Class::kind` / `Label::lang` (absent attribute).
    let kind = attr_value(start, "kind", reader)?.unwrap_or_default();
    let mut super_classes = Vec::new();
    let mut sub_classes = Vec::new();
    let mut rubrics = Vec::new();

    loop {
        match read_event(reader)? {
            Event::End(_) => break,
            Event::Eof => {
                return Err(ClamlError::unexpected_eof(
                    "Class",
                    reader.buffer_position(),
                ));
            }
            Event::Start(e) => match e.local_name().as_ref() {
                b"Rubric" => rubrics.push(parse_rubric(reader, &e)?),
                // A start/end pair (`<SuperClass code="..."></SuperClass>`)
                // is equivalent XML to the usual self-closing form below —
                // record it, then consume through its end tag.
                b"SuperClass" => {
                    if let Some(code) = attr_value(&e, "code", reader)? {
                        super_classes.push(code);
                    }
                    skip_element(reader)?;
                }
                b"SubClass" => {
                    if let Some(code) = attr_value(&e, "code", reader)? {
                        sub_classes.push(code);
                    }
                    skip_element(reader)?;
                }
                _ => skip_element(reader)?,
            },
            Event::Empty(e) => match e.local_name().as_ref() {
                b"SuperClass" => {
                    if let Some(code) = attr_value(&e, "code", reader)? {
                        super_classes.push(code);
                    }
                }
                b"SubClass" => {
                    if let Some(code) = attr_value(&e, "code", reader)? {
                        sub_classes.push(code);
                    }
                }
                b"Rubric" => rubrics.push(finish_rubric(reader, &e)?),
                _ => {}
            },
            _ => {}
        }
    }

    Ok(Class {
        code,
        kind,
        super_classes,
        sub_classes,
        rubrics,
    })
}

/// Builds a `Rubric` with no labels, from a self-closing `<Rubric .../>`
/// tag.
fn finish_rubric(reader: &Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Rubric, ClamlError> {
    Ok(Rubric {
        // RCA-IN-3: absent only; documented on `Class::kind` (absent attribute).
        kind: attr_value(start, "kind", reader)?.unwrap_or_default(),
        labels: Vec::new(),
    })
}

/// Parses a `Rubric` element and its `Label` children, up to its own end
/// tag.
fn parse_rubric(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Rubric, ClamlError> {
    // RCA-IN-3: absent only; documented on `Class::kind` / `Label::lang` (absent attribute).
    let kind = attr_value(start, "kind", reader)?.unwrap_or_default();
    let mut labels = Vec::new();

    loop {
        match read_event(reader)? {
            Event::End(_) => break,
            Event::Eof => {
                return Err(ClamlError::unexpected_eof(
                    "Rubric",
                    reader.buffer_position(),
                ));
            }
            Event::Start(e) => match e.local_name().as_ref() {
                b"Label" => labels.push(parse_label(reader, &e)?),
                _ => skip_element(reader)?,
            },
            Event::Empty(e) if e.local_name().as_ref() == b"Label" => {
                labels.push(finish_label(reader, &e)?);
            }
            _ => {}
        }
    }

    Ok(Rubric { kind, labels })
}

/// Builds a `Label` with empty text, from a self-closing `<Label .../>`
/// tag.
fn finish_label(reader: &Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Label, ClamlError> {
    // RCA-IN-3: absent only; documented on `Class::kind` / `Label::lang` (absent attribute).
    let lang = attr_value(start, "xml:lang", reader)?.unwrap_or_else(|| "en".to_string());
    Ok(Label {
        lang,
        text: String::new(),
    })
}

/// Parses a `Label` element, flattening any nested markup into its text
/// (see [`crate::Label`]'s docs for the rationale).
fn parse_label(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Label, ClamlError> {
    // RCA-IN-3: absent only; documented on `Class::kind` / `Label::lang` (absent attribute).
    let lang = attr_value(start, "xml:lang", reader)?.unwrap_or_else(|| "en".to_string());
    let text = read_flattened_text(reader, "Label")?;
    Ok(Label { lang, text })
}

/// Builds a `ModifierClass` with no children, from a self-closing
/// `<ModifierClass .../>` tag.
fn finish_modifier_class(
    reader: &Reader<&[u8]>,
    start: &BytesStart<'_>,
) -> Result<ModifierClass, ClamlError> {
    let code = attr_value(start, "code", reader)?.ok_or_else(|| {
        ClamlError::missing_attribute("ModifierClass", "code", reader.buffer_position())
    })?;
    Ok(ModifierClass {
        code,
        rubrics: Vec::new(),
        modifiers: Vec::new(),
    })
}

/// Parses a `ModifierClass` element and its `Rubric`/`Modifier` children,
/// up to its own end tag.
fn parse_modifier_class(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
) -> Result<ModifierClass, ClamlError> {
    let code = attr_value(start, "code", reader)?.ok_or_else(|| {
        ClamlError::missing_attribute("ModifierClass", "code", reader.buffer_position())
    })?;
    let mut rubrics = Vec::new();
    let mut modifiers = Vec::new();

    loop {
        match read_event(reader)? {
            Event::End(_) => break,
            Event::Eof => {
                return Err(ClamlError::unexpected_eof(
                    "ModifierClass",
                    reader.buffer_position(),
                ));
            }
            Event::Start(e) => match e.local_name().as_ref() {
                b"Rubric" => rubrics.push(parse_rubric(reader, &e)?),
                b"Modifier" => modifiers.push(parse_modifier(reader, &e)?),
                _ => skip_element(reader)?,
            },
            Event::Empty(e) => match e.local_name().as_ref() {
                b"Rubric" => rubrics.push(finish_rubric(reader, &e)?),
                b"Modifier" => modifiers.push(finish_modifier(reader, &e)?),
                _ => {}
            },
            _ => {}
        }
    }

    Ok(ModifierClass {
        code,
        rubrics,
        modifiers,
    })
}

/// Builds a `Modifier` with no rubrics, from a self-closing
/// `<Modifier .../>` tag.
fn finish_modifier(reader: &Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Modifier, ClamlError> {
    let code = attr_value(start, "code", reader)?.ok_or_else(|| {
        ClamlError::missing_attribute("Modifier", "code", reader.buffer_position())
    })?;
    Ok(Modifier {
        code,
        rubrics: Vec::new(),
    })
}

/// Parses a `Modifier` element and its `Rubric` children, up to its own
/// end tag.
fn parse_modifier(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
) -> Result<Modifier, ClamlError> {
    let code = attr_value(start, "code", reader)?.ok_or_else(|| {
        ClamlError::missing_attribute("Modifier", "code", reader.buffer_position())
    })?;
    let mut rubrics = Vec::new();

    loop {
        match read_event(reader)? {
            Event::End(_) => break,
            Event::Eof => {
                return Err(ClamlError::unexpected_eof(
                    "Modifier",
                    reader.buffer_position(),
                ));
            }
            Event::Start(e) => match e.local_name().as_ref() {
                b"Rubric" => rubrics.push(parse_rubric(reader, &e)?),
                _ => skip_element(reader)?,
            },
            Event::Empty(e) if e.local_name().as_ref() == b"Rubric" => {
                rubrics.push(finish_rubric(reader, &e)?);
            }
            _ => {}
        }
    }

    Ok(Modifier { code, rubrics })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{ClamlDocument, ClamlError};

    fn parse(xml: &str) -> Result<ClamlDocument, ClamlError> {
        ClamlDocument::from_str(xml)
    }

    fn eof_in(xml: &str, element: &str) {
        match parse(xml) {
            Err(ClamlError::UnexpectedEof { element: found, .. }) => assert_eq!(found, element),
            other => panic!("expected UnexpectedEof in {element}, got {other:?}"),
        }
    }

    #[test]
    fn empty_input_and_self_closing_root_are_empty_documents() {
        assert_eq!(parse("").unwrap(), ClamlDocument::default());
        assert_eq!(parse("<ClaML/>").unwrap(), ClamlDocument::default());
        assert_eq!(
            parse("<?xml version=\"1.0\"?><!-- c --><ClaML/>").unwrap(),
            ClamlDocument::default()
        );
    }

    #[test]
    fn truncated_documents_report_the_open_element() {
        eof_in("<ClaML><Class code=\"A\">", "Class");
        eof_in("<ClaML>", "ClaML");
        eof_in("<ClaML><Class code=\"A\"><Rubric kind=\"k\">", "Rubric");
        eof_in("<ClaML><Class code=\"A\"><Rubric><Label>x", "Label");
        eof_in("<ClaML><Title>x", "Title");
        eof_in("<ClaML><ModifierClass code=\"M\">", "ModifierClass");
        eof_in(
            "<ClaML><ModifierClass code=\"M\"><Modifier code=\"m\">",
            "Modifier",
        );
        eof_in("<ClaML><Identifier><a>", "element");
    }

    #[test]
    fn unknown_elements_are_skipped_at_every_level() {
        let xml = "<ClaML><Meta><a><b/></a></Meta><Unknown/>\
            <Class code=\"A\"><Extra><x/></Extra><Unknown/>\
            <Rubric kind=\"k\"><Extra><y/></Extra><Unknown/></Rubric></Class>\
            <ModifierClass code=\"M\"><Extra/><Extra><z/></Extra>\
            <Modifier code=\"m\"><Extra><w/></Extra><Unknown/></Modifier></ModifierClass></ClaML>";
        let doc = parse(xml).unwrap();
        assert_eq!(doc.classes().len(), 1);
        assert_eq!(doc.classes()[0].rubrics().len(), 1);
        assert_eq!(doc.modifier_classes()[0].modifiers().len(), 1);
    }

    #[test]
    fn self_closing_elements_build_empty_values() {
        let xml = "<ClaML><Class code=\"A\" kind=\"block\"/><ModifierClass code=\"M\"/>\
            <Class code=\"B\"><Rubric kind=\"r\"/><SuperClass code=\"A\"/><SubClass code=\"C\"/></Class>\
            <Class code=\"D\"><Rubric kind=\"r\"><Label xml:lang=\"fr\"/><Label/></Rubric></Class>\
            <ModifierClass code=\"N\"><Rubric kind=\"r\"/><Modifier code=\"m\"/></ModifierClass>\
            <Class code=\"E\"><Other/></Class>\
            <ModifierClass code=\"O\"><Other/></ModifierClass>\
            <Other/></ClaML>";
        let doc = parse(xml).unwrap();
        assert_eq!(doc.classes()[0].kind(), "block");
        assert_eq!(doc.classes()[1].super_classes(), ["A"]);
        assert_eq!(doc.classes()[1].sub_classes(), ["C"]);
        let labels = doc.classes()[2].rubrics()[0].labels();
        assert_eq!(labels[0].lang(), "fr");
        assert_eq!(labels[0].text(), "");
        assert_eq!(doc.modifier_classes()[1].modifiers()[0].rubrics().len(), 0);
    }

    #[test]
    fn start_end_pairs_for_super_and_sub_classes_are_recorded() {
        let xml = "<ClaML><Class code=\"A\"><SuperClass code=\"P\"></SuperClass>\
            <SubClass code=\"S\"></SubClass><SuperClass></SuperClass><SubClass></SubClass>\
            <SuperClass/><SubClass/></Class></ClaML>";
        let doc = parse(xml).unwrap();
        assert_eq!(doc.classes()[0].super_classes(), ["P"]);
        assert_eq!(doc.classes()[0].sub_classes(), ["S"]);
    }

    #[test]
    fn entities_cdata_and_nested_markup_flatten_into_label_text() {
        let xml =
            "<ClaML><Title>T &amp; U &#65; &custom;<b>in</b><![CDATA[<raw>]]></Title></ClaML>";
        assert_eq!(parse(xml).unwrap().title(), Some("T & U A &custom;in<raw>"));
    }

    #[test]
    fn missing_code_attributes_are_errors_for_every_element() {
        for (xml, element) in [
            ("<ClaML><Class/></ClaML>", "Class"),
            ("<ClaML><Class></Class></ClaML>", "Class"),
            ("<ClaML><ModifierClass/></ClaML>", "ModifierClass"),
            (
                "<ClaML><ModifierClass></ModifierClass></ClaML>",
                "ModifierClass",
            ),
            (
                "<ClaML><ModifierClass code=\"M\"><Modifier/></ModifierClass></ClaML>",
                "Modifier",
            ),
            (
                "<ClaML><ModifierClass code=\"M\"><Modifier></Modifier></ModifierClass></ClaML>",
                "Modifier",
            ),
        ] {
            match parse(xml) {
                Err(ClamlError::MissingAttribute {
                    element: found,
                    attribute,
                    ..
                }) => {
                    assert_eq!((found, attribute), (element, "code"));
                }
                other => panic!("{xml}: {other:?}"),
            }
        }
    }

    #[test]
    fn malformed_xml_and_bad_attributes_are_errors() {
        for xml in [
            "<ClaML><Class code=\"A\"></Wrong></ClaML>",
            "<ClaML><Class code=\"A\" code=\"B\"/></ClaML>",
            "<ClaML><Class code=A/></ClaML>",
            "<ClaML><Title>&#xZZ;</Title></ClaML>",
            "<ClaML><Title>&#1114112;</Title></ClaML>",
        ] {
            assert!(parse(xml).is_err(), "{xml}");
        }
    }
}
