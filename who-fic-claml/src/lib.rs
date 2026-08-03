//! Parser for ClaML (Classification Markup Language, ISO 13120), the XML
//! format WHO distributes ICD-10 in.
//!
//! This crate is format-only: it has no ICD-10-specific knowledge.
//! `who-fic-icd`'s optional `claml` feature adapts [`ClamlDocument`] into
//! `Icd10Code`-keyed lookups — see `specs/who-fic-claml.md` in the
//! repository for the full format specification and
//! `specs/who-fic-icd.md` for the adapter.
//!
//! This crate parses XML the *user* supplies (obtained under WHO's or the
//! relevant maintainer's own terms); it does not fetch or embed
//! classification content.
//!
//! # Format
//!
//! A ClaML document is a `<ClaML>` root element containing a flat list of
//! `Class` elements. Hierarchy is expressed by reference (a `Class`'s
//! `SuperClass` elements name its parent's `code`), not by XML nesting.
//! Each `Class` carries one or more `Rubric` elements (e.g. the
//! `kind="preferred"` title, `kind="inclusion"` notes, and so on), each of
//! which carries one or more `Label` elements with the actual text.
//!
//! ```
//! use who_fic_claml::ClamlDocument;
//! use std::str::FromStr;
//!
//! let xml = r#"
//! <ClaML version="2.0">
//!   <Title>Sample</Title>
//!   <Class code="A00" kind="category">
//!     <SuperClass code="A00-A09"/>
//!     <Rubric kind="preferred">
//!       <Label xml:lang="en">Cholera</Label>
//!     </Rubric>
//!   </Class>
//! </ClaML>
//! "#;
//!
//! let doc = ClamlDocument::from_str(xml).unwrap();
//! assert_eq!(doc.title(), Some("Sample"));
//! let class = &doc.classes()[0];
//! assert_eq!(class.code(), "A00");
//! assert_eq!(class.preferred_label("en"), Some("Cholera"));
//! ```

#![warn(missing_docs)]

mod parse;

use std::fmt;

/// A fully parsed ClaML document.
///
/// The top-level container for everything a ClaML file describes: an
/// optional title, the flat list of [`Class`] entries, and the flat list
/// of [`ModifierClass`] entries. Hierarchy among classes is expressed by
/// reference ([`Class::super_classes`] / [`Class::sub_classes`]), not by
/// nesting in this struct.
///
/// Construct one by parsing a complete ClaML document with
/// `ClamlDocument::from_str` (via the standard [`std::str::FromStr`]
/// trait) or [`ClamlDocument::from_reader`].
///
/// ```
/// use who_fic_claml::ClamlDocument;
/// use std::str::FromStr;
///
/// let xml = r#"<ClaML version="2.0"><Title>Demo</Title></ClaML>"#;
/// let doc = ClamlDocument::from_str(xml).unwrap();
/// assert_eq!(doc.title(), Some("Demo"));
/// assert!(doc.classes().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClamlDocument {
    pub(crate) title: Option<String>,
    pub(crate) classes: Vec<Class>,
    pub(crate) modifier_classes: Vec<ModifierClass>,
}

impl ClamlDocument {
    /// Parses a complete ClaML document from an in-memory reader.
    ///
    /// This reads the entire stream into memory (as UTF-8 text) before
    /// parsing; for `&str` input already in memory, prefer
    /// `ClamlDocument::from_str` (via [`std::str::FromStr`]), which
    /// avoids the extra copy.
    ///
    /// # Errors
    ///
    /// Returns [`ClamlError`] if the stream cannot be read, is not valid
    /// UTF-8, is not well-formed XML, or does not conform to the
    /// structural requirements this crate checks (e.g. every `Class` must
    /// have a `code` attribute).
    ///
    /// ```
    /// use who_fic_claml::ClamlDocument;
    ///
    /// let xml = br#"<ClaML version="2.0"><Title>Demo</Title></ClaML>"#;
    /// let doc = ClamlDocument::from_reader(&xml[..]).unwrap();
    /// assert_eq!(doc.title(), Some("Demo"));
    /// ```
    pub fn from_reader(mut reader: impl std::io::Read) -> Result<Self, ClamlError> {
        let mut text = String::new();
        reader
            .read_to_string(&mut text)
            .map_err(|source| ClamlError::xml(quick_xml::Error::from(source), 0))?;
        parse::parse_document(&text)
    }

    /// The document's `Title`, if present.
    ///
    /// ```
    /// use who_fic_claml::ClamlDocument;
    /// use std::str::FromStr;
    ///
    /// let doc = ClamlDocument::from_str(r#"<ClaML version="2.0"/>"#).unwrap();
    /// assert_eq!(doc.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The flat list of `Class` entries in document order.
    pub fn classes(&self) -> &[Class] {
        &self.classes
    }

    /// The flat list of `ModifierClass` entries in document order.
    pub fn modifier_classes(&self) -> &[ModifierClass] {
        &self.modifier_classes
    }
}

impl std::str::FromStr for ClamlDocument {
    type Err = ClamlError;

    /// Parses a complete ClaML document from a string already in memory.
    ///
    /// ```
    /// use who_fic_claml::ClamlDocument;
    /// use std::str::FromStr;
    ///
    /// let doc = ClamlDocument::from_str(r#"<ClaML version="2.0"/>"#).unwrap();
    /// assert!(doc.classes().is_empty());
    /// ```
    fn from_str(xml: &str) -> Result<Self, Self::Err> {
        parse::parse_document(xml)
    }
}

/// One `Class` element: a single entry in the classification (e.g. a
/// chapter, block, or category).
///
/// A `Class` names its position in the hierarchy by reference rather than
/// by XML nesting: [`Class::super_classes`] lists the codes of its
/// parent(s) (normally zero, for root-level chapters, or one; the DTD
/// permits more), and [`Class::sub_classes`] is a redundant
/// forward-reference list of its children's codes, kept for convenience
/// but not authoritative — [`Class::super_classes`] is the source of truth
/// for hierarchy.
///
/// ```
/// use who_fic_claml::ClamlDocument;
/// use std::str::FromStr;
///
/// let xml = r#"
/// <ClaML version="2.0">
///   <Class code="A00" kind="category">
///     <SuperClass code="A00-A09"/>
///     <Rubric kind="preferred"><Label xml:lang="en">Cholera</Label></Rubric>
///   </Class>
/// </ClaML>
/// "#;
/// let doc = ClamlDocument::from_str(xml).unwrap();
/// let class = &doc.classes()[0];
/// assert_eq!(class.code(), "A00");
/// assert_eq!(class.kind(), "category");
/// assert_eq!(class.super_classes(), ["A00-A09"]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Class {
    pub(crate) code: String,
    pub(crate) kind: String,
    pub(crate) super_classes: Vec<String>,
    pub(crate) sub_classes: Vec<String>,
    pub(crate) rubrics: Vec<Rubric>,
}

impl Class {
    /// The classification code, e.g. `"A00"`.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The class kind, e.g. `"chapter"`, `"block"`, `"category"`.
    ///
    /// The set of kinds is open-ended, defined by the source file's own
    /// `ClassKinds` declaration; this crate does not parse or validate
    /// against that declaration, it just captures whatever string is on
    /// the attribute. Empty (`""`) if the `Class` element has no `kind`
    /// attribute.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// The codes of this class's parent(s), from its `SuperClass`
    /// elements, in document order.
    ///
    /// Normally zero (root-level chapters) or one element; the ClaML DTD
    /// permits more than one for classifications with multiple
    /// inheritance, so all present `SuperClass` elements are collected.
    pub fn super_classes(&self) -> &[String] {
        &self.super_classes
    }

    /// The codes of this class's children, from its `SubClass` elements,
    /// in document order.
    ///
    /// This is a redundant forward-reference list kept by ClaML for
    /// convenience; it is derivable from other classes'
    /// [`Class::super_classes`] and is not treated as more authoritative.
    pub fn sub_classes(&self) -> &[String] {
        &self.sub_classes
    }

    /// This class's `Rubric` elements, in document order.
    pub fn rubrics(&self) -> &[Rubric] {
        &self.rubrics
    }

    /// The text of this class's `kind="preferred"` rubric, in the given
    /// language.
    ///
    /// Returns `None` if there is no preferred rubric, or none of its
    /// labels match `lang`.
    ///
    /// ```
    /// use who_fic_claml::ClamlDocument;
    /// use std::str::FromStr;
    ///
    /// let xml = r#"
    /// <ClaML version="2.0">
    ///   <Class code="A00" kind="category">
    ///     <Rubric kind="preferred"><Label xml:lang="en">Cholera</Label></Rubric>
    ///     <Rubric kind="inclusion"><Label xml:lang="en">Cholera due to Vibrio cholerae</Label></Rubric>
    ///   </Class>
    /// </ClaML>
    /// "#;
    /// let doc = ClamlDocument::from_str(xml).unwrap();
    /// let class = &doc.classes()[0];
    /// assert_eq!(class.preferred_label("en"), Some("Cholera"));
    /// assert_eq!(class.preferred_label("fr"), None);
    /// ```
    pub fn preferred_label(&self, lang: &str) -> Option<&str> {
        self.rubrics
            .iter()
            .find(|rubric| rubric.kind == "preferred")
            .and_then(|rubric| rubric.labels.iter().find(|label| label.lang == lang))
            .map(|label| label.text.as_str())
    }
}

/// A `Rubric` element: a named group of [`Label`]s attached to a [`Class`],
/// [`ModifierClass`], or [`Modifier`].
///
/// `kind="preferred"` is the canonical title; other kinds observed in
/// practice include `"inclusion"`, `"exclusion"`, `"definition"`, and
/// `"coding-hint"`. The set of kinds is classification-defined, not fixed
/// by the ClaML format, so this crate captures `kind` generically as a
/// string rather than a fixed enum.
///
/// ```
/// use who_fic_claml::ClamlDocument;
/// use std::str::FromStr;
///
/// let xml = r#"
/// <ClaML version="2.0">
///   <Class code="A00" kind="category">
///     <Rubric kind="inclusion"><Label xml:lang="en">Cholera due to Vibrio cholerae</Label></Rubric>
///   </Class>
/// </ClaML>
/// "#;
/// let doc = ClamlDocument::from_str(xml).unwrap();
/// let rubric = &doc.classes()[0].rubrics()[0];
/// assert_eq!(rubric.kind(), "inclusion");
/// assert_eq!(rubric.labels()[0].text(), "Cholera due to Vibrio cholerae");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rubric {
    pub(crate) kind: String,
    pub(crate) labels: Vec<Label>,
}

impl Rubric {
    /// The rubric kind, e.g. `"preferred"`, `"inclusion"`, `"exclusion"`.
    ///
    /// Empty (`""`) if the `Rubric` element has no `kind` attribute.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// This rubric's `Label` elements, in document order.
    pub fn labels(&self) -> &[Label] {
        &self.labels
    }
}

/// A single `Label`: text in one language, attached to a [`Rubric`].
///
/// **Simplification:** real ClaML files can nest simple inline markup
/// inside a `Label` (e.g. `Para` paragraphs or cross-reference elements).
/// This crate does not preserve that markup structure — it concatenates
/// all text nodes found anywhere within the `Label` element into a single
/// string and discards the surrounding tags. This is a deliberate
/// simplification suited to this crate's scope (extracting classification
/// text), not a full ClaML content model.
///
/// ```
/// use who_fic_claml::ClamlDocument;
/// use std::str::FromStr;
///
/// let xml = r#"
/// <ClaML version="2.0">
///   <Class code="A00" kind="category">
///     <Rubric kind="preferred"><Label xml:lang="en">Cholera</Label></Rubric>
///   </Class>
/// </ClaML>
/// "#;
/// let doc = ClamlDocument::from_str(xml).unwrap();
/// let label = &doc.classes()[0].rubrics()[0].labels()[0];
/// assert_eq!(label.lang(), "en");
/// assert_eq!(label.text(), "Cholera");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
    pub(crate) lang: String,
    pub(crate) text: String,
}

impl Label {
    /// The label's language, from its `xml:lang` attribute.
    ///
    /// Defaults to `"en"` when the attribute is absent, since WHO's
    /// ClaML exports are English-only in practice.
    pub fn lang(&self) -> &str {
        &self.lang
    }

    /// The label's concatenated text content (see the type-level docs for
    /// how nested markup is flattened).
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// A `ModifierClass` element: a shared subclassification scheme (e.g. a
/// 4th- or 5th-character modifier list shared by several categories).
///
/// Parsed into a generic form with no ICD-10-specific modifier semantics;
/// that interpretation belongs in `who-fic-icd` if implemented.
///
/// ```
/// use who_fic_claml::ClamlDocument;
/// use std::str::FromStr;
///
/// let xml = r#"
/// <ClaML version="2.0">
///   <ModifierClass code="M1">
///     <Rubric kind="preferred"><Label xml:lang="en">General fifth-character modifiers</Label></Rubric>
///     <Modifier code="0">
///       <Rubric kind="preferred"><Label xml:lang="en">Without mention of complication</Label></Rubric>
///     </Modifier>
///   </ModifierClass>
/// </ClaML>
/// "#;
/// let doc = ClamlDocument::from_str(xml).unwrap();
/// let modifier_class = &doc.modifier_classes()[0];
/// assert_eq!(modifier_class.code(), "M1");
/// assert_eq!(modifier_class.modifiers()[0].code(), "0");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifierClass {
    pub(crate) code: String,
    pub(crate) rubrics: Vec<Rubric>,
    pub(crate) modifiers: Vec<Modifier>,
}

impl ModifierClass {
    /// The modifier class's code, e.g. `"M1"`.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// This modifier class's own `Rubric` elements (e.g. its title), in
    /// document order.
    pub fn rubrics(&self) -> &[Rubric] {
        &self.rubrics
    }

    /// This modifier class's `Modifier` entries, in document order.
    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }
}

/// A single `Modifier` entry within a [`ModifierClass`].
///
/// ```
/// use who_fic_claml::ClamlDocument;
/// use std::str::FromStr;
///
/// let xml = r#"
/// <ClaML version="2.0">
///   <ModifierClass code="M1">
///     <Modifier code="1">
///       <Rubric kind="preferred"><Label xml:lang="en">With complication</Label></Rubric>
///     </Modifier>
///   </ModifierClass>
/// </ClaML>
/// "#;
/// let doc = ClamlDocument::from_str(xml).unwrap();
/// let modifier = &doc.modifier_classes()[0].modifiers()[0];
/// assert_eq!(modifier.code(), "1");
/// assert_eq!(modifier.rubrics()[0].labels()[0].text(), "With complication");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Modifier {
    pub(crate) code: String,
    pub(crate) rubrics: Vec<Rubric>,
}

impl Modifier {
    /// The modifier's code, e.g. `"0"`.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// This modifier's `Rubric` elements, in document order.
    pub fn rubrics(&self) -> &[Rubric] {
        &self.rubrics
    }
}

/// An error encountered while parsing a ClaML document.
///
/// Wraps both underlying XML parse errors (reported by `quick-xml`, with
/// byte-offset position information where available) and this crate's own
/// structural checks (e.g. a `Class` element missing its required `code`
/// attribute).
///
/// `#[non_exhaustive]` so new structural checks can be added without a
/// breaking change.
///
/// ```
/// use who_fic_claml::ClamlDocument;
/// use std::str::FromStr;
///
/// // A `Class` with no `code` attribute is structurally invalid ClaML.
/// let xml = r#"<ClaML version="2.0"><Class kind="category"/></ClaML>"#;
/// let err = ClamlDocument::from_str(xml).unwrap_err();
/// assert!(err.to_string().contains("code"));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ClamlError {
    /// The input was not well-formed XML (or could not be read/decoded as
    /// UTF-8 text), as reported by the underlying `quick-xml` parser.
    Xml {
        /// The underlying `quick-xml` error.
        source: quick_xml::Error,
        /// Byte offset into the input where the error was detected, when
        /// known.
        position: u64,
    },
    /// A required attribute was missing from an element, e.g. a `Class`
    /// with no `code` attribute.
    MissingAttribute {
        /// The element's tag name, e.g. `"Class"`.
        element: &'static str,
        /// The missing attribute's name, e.g. `"code"`.
        attribute: &'static str,
        /// Byte offset into the input where the element containing the
        /// error ends.
        position: u64,
    },
    /// The document ended before a required closing tag was found.
    UnexpectedEof {
        /// The tag name of the element that was left unclosed.
        element: &'static str,
        /// Byte offset into the input where parsing stopped.
        position: u64,
    },
}

impl ClamlError {
    pub(crate) fn xml(source: quick_xml::Error, position: u64) -> Self {
        ClamlError::Xml { source, position }
    }

    pub(crate) fn missing_attribute(
        element: &'static str,
        attribute: &'static str,
        position: u64,
    ) -> Self {
        ClamlError::MissingAttribute {
            element,
            attribute,
            position,
        }
    }

    pub(crate) fn unexpected_eof(element: &'static str, position: u64) -> Self {
        ClamlError::UnexpectedEof { element, position }
    }
}

impl fmt::Display for ClamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClamlError::Xml { source, position } => {
                write!(f, "XML parse error at byte {position}: {source}")
            }
            ClamlError::MissingAttribute {
                element,
                attribute,
                position,
            } => write!(
                f,
                "<{element}> is missing its required `{attribute}` attribute (near byte {position})"
            ),
            ClamlError::UnexpectedEof { element, position } => write!(
                f,
                "unexpected end of document inside <{element}> (near byte {position})"
            ),
        }
    }
}

impl std::error::Error for ClamlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClamlError::Xml { source, .. } => Some(source),
            ClamlError::MissingAttribute { .. } | ClamlError::UnexpectedEof { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// A small hand-written fixture in the real ClaML shape, using ICD-10
    /// codes already used as examples elsewhere in this workspace (`A00`
    /// et al.) — not vendored WHO content.
    const FIXTURE: &str = r#"
<ClaML version="2.0">
  <Title>Sample</Title>
  <Class code="A00-A09" kind="block">
    <SuperClass code="A00-B99"/>
    <SubClass code="A00"/>
    <SubClass code="A01"/>
    <Rubric kind="preferred">
      <Label xml:lang="en">Intestinal infectious diseases</Label>
    </Rubric>
  </Class>
  <Class code="A00" kind="category">
    <SuperClass code="A00-A09"/>
    <Rubric kind="preferred">
      <Label xml:lang="en">Cholera</Label>
    </Rubric>
    <Rubric kind="inclusion">
      <Label xml:lang="en">Cholera due to Vibrio cholerae</Label>
    </Rubric>
    <Rubric kind="exclusion">
      <Label xml:lang="en">Cholera carrier</Label>
    </Rubric>
  </Class>
  <ModifierClass code="M1">
    <Rubric kind="preferred">
      <Label xml:lang="en">General fifth-character modifiers</Label>
    </Rubric>
    <Modifier code="0">
      <Rubric kind="preferred">
        <Label xml:lang="en">Without mention of complication</Label>
      </Rubric>
    </Modifier>
    <Modifier code="1">
      <Rubric kind="preferred">
        <Label xml:lang="en">With complication</Label>
      </Rubric>
    </Modifier>
  </ModifierClass>
</ClaML>
"#;

    fn fixture() -> ClamlDocument {
        ClamlDocument::from_str(FIXTURE).expect("fixture should parse")
    }

    fn find_class<'a>(doc: &'a ClamlDocument, code: &str) -> &'a Class {
        doc.classes()
            .iter()
            .find(|c| c.code() == code)
            .unwrap_or_else(|| panic!("no class with code {code:?}"))
    }

    #[test]
    fn parses_document_title() {
        let doc = fixture();
        assert_eq!(doc.title(), Some("Sample"));
    }

    #[test]
    fn parses_class_with_preferred_and_inclusion_rubric() {
        let doc = fixture();
        let a00 = find_class(&doc, "A00");
        assert_eq!(a00.kind(), "category");
        assert_eq!(a00.preferred_label("en"), Some("Cholera"));

        let kinds: Vec<&str> = a00.rubrics().iter().map(Rubric::kind).collect();
        assert_eq!(kinds, ["preferred", "inclusion", "exclusion"]);

        let inclusion = a00
            .rubrics()
            .iter()
            .find(|r| r.kind() == "inclusion")
            .unwrap();
        assert_eq!(
            inclusion.labels()[0].text(),
            "Cholera due to Vibrio cholerae"
        );
        assert_eq!(inclusion.labels()[0].lang(), "en");

        let exclusion = a00
            .rubrics()
            .iter()
            .find(|r| r.kind() == "exclusion")
            .unwrap();
        assert_eq!(exclusion.labels()[0].text(), "Cholera carrier");
    }

    #[test]
    fn parses_class_with_multiple_sub_classes() {
        let doc = fixture();
        let block = find_class(&doc, "A00-A09");
        assert_eq!(block.sub_classes(), ["A00", "A01"]);
        assert_eq!(block.super_classes(), ["A00-B99"]);
        assert_eq!(block.kind(), "block");
    }

    #[test]
    fn parses_class_with_more_than_one_super_class() {
        let xml = r#"
<ClaML version="2.0">
  <Class code="X99" kind="category">
    <SuperClass code="X00-X49"/>
    <SuperClass code="X50-X99"/>
    <Rubric kind="preferred"><Label xml:lang="en">Dual-parented example</Label></Rubric>
  </Class>
</ClaML>
"#;
        let doc = ClamlDocument::from_str(xml).unwrap();
        let class = &doc.classes()[0];
        assert_eq!(class.super_classes(), ["X00-X49", "X50-X99"]);
    }

    #[test]
    fn parses_modifier_class_and_modifiers() {
        let doc = fixture();
        assert_eq!(doc.modifier_classes().len(), 1);
        let m1 = &doc.modifier_classes()[0];
        assert_eq!(m1.code(), "M1");
        assert_eq!(
            m1.rubrics()[0].labels()[0].text(),
            "General fifth-character modifiers"
        );
        assert_eq!(m1.modifiers().len(), 2);
        assert_eq!(m1.modifiers()[0].code(), "0");
        assert_eq!(
            m1.modifiers()[0].rubrics()[0].labels()[0].text(),
            "Without mention of complication"
        );
        assert_eq!(m1.modifiers()[1].code(), "1");
        assert_eq!(
            m1.modifiers()[1].rubrics()[0].labels()[0].text(),
            "With complication"
        );
    }

    #[test]
    fn malformed_xml_unclosed_tag_returns_error() {
        // The `<Class` start tag is never closed with `>`.
        let xml = r#"<ClaML version="2.0"><Class code="A00""#;
        let err = ClamlDocument::from_str(xml).unwrap_err();
        assert!(matches!(err, ClamlError::Xml { .. }), "got: {err:?}");
    }

    #[test]
    fn class_missing_code_attribute_returns_error() {
        let xml = r#"<ClaML version="2.0"><Class kind="category"/></ClaML>"#;
        let err = ClamlDocument::from_str(xml).unwrap_err();
        match err {
            ClamlError::MissingAttribute {
                element, attribute, ..
            } => {
                assert_eq!(element, "Class");
                assert_eq!(attribute, "code");
            }
            other => panic!("expected MissingAttribute, got {other:?}"),
        }
    }

    #[test]
    fn modifier_missing_code_attribute_returns_error() {
        let xml =
            r#"<ClaML version="2.0"><ModifierClass code="M1"><Modifier/></ModifierClass></ClaML>"#;
        let err = ClamlDocument::from_str(xml).unwrap_err();
        match err {
            ClamlError::MissingAttribute {
                element, attribute, ..
            } => {
                assert_eq!(element, "Modifier");
                assert_eq!(attribute, "code");
            }
            other => panic!("expected MissingAttribute, got {other:?}"),
        }
    }

    #[test]
    fn label_defaults_lang_to_en_when_absent() {
        let xml = r#"
<ClaML version="2.0">
  <Class code="A00" kind="category">
    <Rubric kind="preferred"><Label>Cholera</Label></Rubric>
  </Class>
</ClaML>
"#;
        let doc = ClamlDocument::from_str(xml).unwrap();
        assert_eq!(doc.classes()[0].rubrics()[0].labels()[0].lang(), "en");
    }

    #[test]
    fn label_flattens_nested_markup_to_text() {
        let xml = r#"
<ClaML version="2.0">
  <Class code="A00" kind="category">
    <Rubric kind="definition">
      <Label xml:lang="en">An <Para>acute</Para> diarrhoeal illness</Label>
    </Rubric>
  </Class>
</ClaML>
"#;
        let doc = ClamlDocument::from_str(xml).unwrap();
        let text = doc.classes()[0].rubrics()[0].labels()[0].text();
        assert_eq!(text, "An acute diarrhoeal illness");
    }

    #[test]
    fn from_reader_matches_from_str() {
        let by_str = ClamlDocument::from_str(FIXTURE).unwrap();
        let by_reader = ClamlDocument::from_reader(FIXTURE.as_bytes()).unwrap();
        assert_eq!(by_str, by_reader);
    }

    #[test]
    fn empty_document_has_no_title_or_classes() {
        let doc = ClamlDocument::from_str(r#"<ClaML version="2.0"/>"#).unwrap();
        assert_eq!(doc.title(), None);
        assert!(doc.classes().is_empty());
        assert!(doc.modifier_classes().is_empty());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trips_through_json() {
        let doc = fixture();
        let json = serde_json::to_string(&doc).expect("serialize");
        let round_tripped: ClamlDocument = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(doc, round_tripped);
    }
}
