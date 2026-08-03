//! ICD-10: the Tenth Revision of the International Classification of
//! Diseases.
//!
//! # Code syntax
//!
//! ```text
//! code        = category [ "." subdivision ]
//! category    = letter digit digit            ; "A00" … "Z99"
//! subdivision = 1*2( digit / letter )         ; commonly one digit, e.g. "I63.9"
//! ```
//!
//! # Out of scope
//!
//! This parser deliberately does **not** support:
//!
//! - Dagger (`†`) and asterisk (`*`) dual-classification markers.
//! - National clinical modifications with a 7-character extension digit
//!   (e.g. ICD-10-CM codes such as `S72.001A`). Any input longer than a
//!   3-character category plus a 1-2 character subdivision is rejected
//!   with [`Icd10ParseError::InvalidLength`].
//!
//! Parsing validates syntax and structure only, not whether WHO has
//! actually assigned a given code (existence requires WHO data and is out
//! of scope for this crate — see `specs/architecture.md`).

use std::fmt;
use std::str::FromStr;

/// An ICD-10 code, e.g. `A00` or `I63.9`.
///
/// Wraps a validated, canonical (uppercase) string form. The only way to
/// construct one is by parsing (`FromStr`/`TryFrom<&str>`) — there is no
/// public constructor that skips validation.
///
/// Parsing is case-insensitive; the canonical form (returned by
/// [`Icd10Code::as_str`] and `Display`) is always uppercase.
///
/// ```
/// use who_fic_icd::icd10::Icd10Code;
///
/// let code: Icd10Code = "i63.9".parse().unwrap();
/// assert_eq!(code.as_str(), "I63.9");
/// assert_eq!(code.to_string(), "I63.9");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Icd10Code(String);

impl Icd10Code {
    /// Returns the canonical (uppercase) string form of the code.
    ///
    /// ```
    /// use who_fic_icd::icd10::Icd10Code;
    ///
    /// let code: Icd10Code = "a00".parse().unwrap();
    /// assert_eq!(code.as_str(), "A00");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the 3-character category, e.g. `"I63"` for `I63.9`.
    ///
    /// ```
    /// use who_fic_icd::icd10::Icd10Code;
    ///
    /// let code: Icd10Code = "I63.9".parse().unwrap();
    /// assert_eq!(code.category(), "I63");
    /// ```
    pub fn category(&self) -> &str {
        &self.0[0..3]
    }

    /// Returns the subdivision (the part after the `.`), if present.
    ///
    /// ```
    /// use who_fic_icd::icd10::Icd10Code;
    ///
    /// let with_sub: Icd10Code = "I63.9".parse().unwrap();
    /// assert_eq!(with_sub.subdivision(), Some("9"));
    ///
    /// let without_sub: Icd10Code = "A00".parse().unwrap();
    /// assert_eq!(without_sub.subdivision(), None);
    /// ```
    pub fn subdivision(&self) -> Option<&str> {
        if self.0.len() > 3 {
            Some(&self.0[4..])
        } else {
            None
        }
    }

    /// Returns the chapter containing this code's category, or `None` if
    /// the category falls in a range WHO has reserved but not yet
    /// assigned to any chapter (e.g. `D49`, `T99`, `V00`).
    ///
    /// ```
    /// use who_fic_icd::icd10::{Icd10Chapter, Icd10Code};
    ///
    /// let code: Icd10Code = "I63.9".parse().unwrap();
    /// assert_eq!(code.chapter(), Some(Icd10Chapter::IX));
    /// ```
    pub fn chapter(&self) -> Option<Icd10Chapter> {
        Icd10Chapter::from_category(self.category())
    }
}

impl fmt::Display for Icd10Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Icd10Code {
    type Err = Icd10ParseError;

    /// Parses an ICD-10 code. Case-insensitive; whitespace is not trimmed.
    ///
    /// ```
    /// use who_fic_icd::icd10::Icd10Code;
    /// use std::str::FromStr;
    ///
    /// assert!(Icd10Code::from_str("A00").is_ok());
    /// assert!(Icd10Code::from_str("").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Icd10ParseError::Empty);
        }
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        if len < 3 {
            return Err(Icd10ParseError::InvalidLength { found: len });
        }

        let letter = chars[0];
        if !letter.is_ascii_alphabetic() {
            return Err(Icd10ParseError::InvalidCharacter {
                position: 0,
                found: letter,
            });
        }
        for (offset, c) in chars[1..3].iter().enumerate() {
            if !c.is_ascii_digit() {
                return Err(Icd10ParseError::InvalidCharacter {
                    position: 1 + offset,
                    found: *c,
                });
            }
        }

        let mut canonical = String::with_capacity(len);
        canonical.push(letter.to_ascii_uppercase());
        canonical.push(chars[1]);
        canonical.push(chars[2]);

        if len == 3 {
            return Ok(Icd10Code(canonical));
        }
        if len > 6 {
            return Err(Icd10ParseError::InvalidLength { found: len });
        }
        if chars[3] != '.' {
            return Err(Icd10ParseError::InvalidCharacter {
                position: 3,
                found: chars[3],
            });
        }
        let subdivision = &chars[4..];
        if subdivision.is_empty() {
            return Err(Icd10ParseError::InvalidStructure {
                reason: "subdivision must have 1 to 2 characters after '.'",
            });
        }
        canonical.push('.');
        for (offset, c) in subdivision.iter().enumerate() {
            if !c.is_ascii_alphanumeric() {
                return Err(Icd10ParseError::InvalidCharacter {
                    position: 4 + offset,
                    found: *c,
                });
            }
            canonical.push(c.to_ascii_uppercase());
        }
        Ok(Icd10Code(canonical))
    }
}

impl TryFrom<&str> for Icd10Code {
    type Error = Icd10ParseError;

    /// ```
    /// use who_fic_icd::icd10::Icd10Code;
    ///
    /// let code = Icd10Code::try_from("u07.1").unwrap();
    /// assert_eq!(code.as_str(), "U07.1");
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Icd10Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Icd10Code {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// The 22 official chapters of ICD-10, identified by category range.
///
/// Chapters are **not** aligned to single letters: for example chapter II
/// (`C00`–`D48`) and chapter III (`D50`–`D89`) both use the letter `D`, and
/// chapter XX (`V01`–`Y98`) spans four letters (`V`, `W`, `X`, `Y`).
///
/// This is WHO's official chapter-to-category-range table (a structural
/// fact, not copyrighted classification content):
///
/// | Chapter | Range     | Chapter | Range     |
/// |---------|-----------|---------|-----------|
/// | I       | A00–B99   | XII     | L00–L99   |
/// | II      | C00–D48   | XIII    | M00–M99   |
/// | III     | D50–D89   | XIV     | N00–N99   |
/// | IV      | E00–E90   | XV      | O00–O99   |
/// | V       | F00–F99   | XVI     | P00–P96   |
/// | VI      | G00–G99   | XVII    | Q00–Q99   |
/// | VII     | H00–H59   | XVIII   | R00–R99   |
/// | VIII    | H60–H95   | XIX     | S00–T98   |
/// | IX      | I00–I99   | XX      | V01–Y98   |
/// | X       | J00–J99   | XXI     | Z00–Z99   |
/// | XI      | K00–K93   | XXII    | U00–U99   |
///
/// A handful of categories within these letters are reserved by WHO for
/// future use and are not part of any chapter (e.g. `D49`, `E91`–`E99`,
/// `K94`–`K99`, `P97`–`P99`, `T99`, `V00`, `Y99`); [`Icd10Code::chapter`]
/// returns `None` for those.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Icd10Chapter {
    /// I: Certain infectious and parasitic diseases (A00–B99).
    I,
    /// II: Neoplasms (C00–D48).
    II,
    /// III: Diseases of the blood and blood-forming organs and certain
    /// disorders involving the immune mechanism (D50–D89).
    III,
    /// IV: Endocrine, nutritional and metabolic diseases (E00–E90).
    IV,
    /// V: Mental and behavioural disorders (F00–F99).
    V,
    /// VI: Diseases of the nervous system (G00–G99).
    VI,
    /// VII: Diseases of the eye and adnexa (H00–H59).
    VII,
    /// VIII: Diseases of the ear and mastoid process (H60–H95).
    VIII,
    /// IX: Diseases of the circulatory system (I00–I99).
    IX,
    /// X: Diseases of the respiratory system (J00–J99).
    X,
    /// XI: Diseases of the digestive system (K00–K93).
    XI,
    /// XII: Diseases of the skin and subcutaneous tissue (L00–L99).
    XII,
    /// XIII: Diseases of the musculoskeletal system and connective tissue
    /// (M00–M99).
    XIII,
    /// XIV: Diseases of the genitourinary system (N00–N99).
    XIV,
    /// XV: Pregnancy, childbirth and the puerperium (O00–O99).
    XV,
    /// XVI: Certain conditions originating in the perinatal period
    /// (P00–P96).
    XVI,
    /// XVII: Congenital malformations, deformations and chromosomal
    /// abnormalities (Q00–Q99).
    XVII,
    /// XVIII: Symptoms, signs and abnormal clinical and laboratory
    /// findings, not elsewhere classified (R00–R99).
    XVIII,
    /// XIX: Injury, poisoning and certain other consequences of external
    /// causes (S00–T98).
    XIX,
    /// XX: External causes of morbidity and mortality (V01–Y98).
    XX,
    /// XXI: Factors influencing health status and contact with health
    /// services (Z00–Z99).
    XXI,
    /// XXII: Codes for special purposes (U00–U99).
    XXII,
}

impl Icd10Chapter {
    /// Returns the chapter containing the given 3-character category
    /// (e.g. `"A00"`), or `None` if the category is not a valid 3-character
    /// `letter digit digit` category, or falls in a range WHO has reserved
    /// but not assigned to a chapter.
    ///
    /// ```
    /// use who_fic_icd::icd10::Icd10Chapter;
    ///
    /// assert_eq!(Icd10Chapter::from_category("D50"), Some(Icd10Chapter::III));
    /// assert_eq!(Icd10Chapter::from_category("D49"), None); // reserved, unassigned
    /// ```
    pub fn from_category(category: &str) -> Option<Self> {
        let bytes = category.as_bytes();
        if bytes.len() != 3 {
            return None;
        }
        let letter = bytes[0].to_ascii_uppercase();
        if !letter.is_ascii_uppercase() {
            return None;
        }
        if !bytes[1].is_ascii_digit() || !bytes[2].is_ascii_digit() {
            return None;
        }
        let num = u32::from(bytes[1] - b'0') * 10 + u32::from(bytes[2] - b'0');
        let ordinal = u32::from(letter - b'A') * 100 + num;
        Self::from_ordinal(ordinal)
    }

    fn from_ordinal(ordinal: u32) -> Option<Self> {
        use Icd10Chapter::*;
        match ordinal {
            0..=199 => Some(I),
            200..=348 => Some(II),
            350..=389 => Some(III),
            400..=490 => Some(IV),
            500..=599 => Some(V),
            600..=699 => Some(VI),
            700..=759 => Some(VII),
            760..=795 => Some(VIII),
            800..=899 => Some(IX),
            900..=999 => Some(X),
            1000..=1093 => Some(XI),
            1100..=1199 => Some(XII),
            1200..=1299 => Some(XIII),
            1300..=1399 => Some(XIV),
            1400..=1499 => Some(XV),
            1500..=1596 => Some(XVI),
            1600..=1699 => Some(XVII),
            1700..=1799 => Some(XVIII),
            1800..=1998 => Some(XIX),
            2000..=2099 => Some(XXII),
            2101..=2498 => Some(XX),
            2500..=2599 => Some(XXI),
            _ => None,
        }
    }
}

/// An error parsing an [`Icd10Code`].
///
/// Follows the shared error shape used across `who-fic` crates (see
/// `specs/architecture.md`): empty input, invalid length, an invalid
/// character at a known position, or a structural problem that isn't
/// reducible to a single bad character or length.
///
/// ```
/// use who_fic_icd::icd10::{Icd10Code, Icd10ParseError};
/// use std::str::FromStr;
///
/// assert_eq!(Icd10Code::from_str(""), Err(Icd10ParseError::Empty));
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Icd10ParseError {
    /// The input was empty.
    Empty,
    /// The input's overall length is not valid for any ICD-10 code shape.
    InvalidLength {
        /// The number of characters found.
        found: usize,
    },
    /// A specific character was not valid at its position.
    InvalidCharacter {
        /// The zero-based character position of the offending character.
        position: usize,
        /// The offending character.
        found: char,
    },
    /// The input had a structural problem not reducible to a single bad
    /// character or an invalid overall length.
    InvalidStructure {
        /// A human-readable explanation.
        reason: &'static str,
    },
}

impl fmt::Display for Icd10ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "ICD-10 code is empty"),
            Self::InvalidLength { found } => {
                write!(f, "invalid ICD-10 code length: {found}")
            }
            Self::InvalidCharacter { position, found } => {
                write!(f, "invalid character {found:?} at position {position}")
            }
            Self::InvalidStructure { reason } => {
                write!(f, "invalid ICD-10 code structure: {reason}")
            }
        }
    }
}

impl std::error::Error for Icd10ParseError {}

/// Adapts parsed ClaML (`who-fic-claml`) documents into [`Icd10Code`]-keyed
/// title lookups.
///
/// Requires the `claml` feature. WHO distributes ICD-10 as ClaML XML; per
/// this workspace's licensing constraint (`specs/architecture.md`), this
/// crate never bundles or fetches that content — the caller parses a ClaML
/// export they obtained themselves into a
/// [`who_fic_claml::ClamlDocument`], and [`claml::Icd10ClamlIndex::from_document`]
/// adapts its generic `Class` list into typed [`Icd10Code`] lookups.
///
/// A real ClaML document mixes ICD-10 category classes (e.g.
/// `code="A00"`) with chapter and block classes whose `code` is a *range*
/// (e.g. `"A00-A09"`), which is not valid single-code ICD-10 syntax. That
/// is expected and normal, not a parse failure of the document: classes
/// whose `code` does not parse as an [`Icd10Code`] are silently skipped
/// rather than treated as an error, and only ICD-10-shaped classes end up
/// in the index.
#[cfg(feature = "claml")]
pub mod claml {
    use super::{Icd10Code, Icd10ParseError};
    use std::collections::HashMap;
    use std::fmt;
    use std::str::FromStr;
    use who_fic_claml::ClamlDocument;

    /// One ClaML `Class` indexed by [`Icd10ClamlIndex`], adapted to a typed
    /// [`Icd10Code`].
    ///
    /// ```
    /// use who_fic_claml::ClamlDocument;
    /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
    /// use who_fic_icd::icd10::Icd10Code;
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
    /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
    /// let entry = index.get(&Icd10Code::from_str("A00").unwrap()).unwrap();
    /// assert_eq!(entry.code().as_str(), "A00");
    /// assert_eq!(entry.title(), "Cholera");
    /// assert_eq!(entry.kind(), "category");
    /// ```
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct Icd10ClassEntry {
        code: Icd10Code,
        title: String,
        kind: String,
    }

    impl Icd10ClassEntry {
        /// The entry's ICD-10 code.
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use who_fic_icd::icd10::Icd10Code;
        /// use std::str::FromStr;
        ///
        /// let xml = r#"<ClaML version="2.0"><Class code="A00" kind="category"/></ClaML>"#;
        /// let doc = ClamlDocument::from_str(xml).unwrap();
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// let entry = index.get(&Icd10Code::from_str("A00").unwrap()).unwrap();
        /// assert_eq!(entry.code(), &Icd10Code::from_str("A00").unwrap());
        /// ```
        pub fn code(&self) -> &Icd10Code {
            &self.code
        }

        /// The class's preferred English (`xml:lang="en"`) title, from its
        /// `kind="preferred"` `Rubric`.
        ///
        /// Empty (`""`) if the class had no preferred-English rubric —
        /// this module's documented policy is to index every
        /// ICD-10-shaped class it finds and default a missing title to
        /// empty, rather than dropping the entry (the caller may still
        /// want the entry for its `kind`, even without a title).
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use who_fic_icd::icd10::Icd10Code;
        /// use std::str::FromStr;
        ///
        /// let xml = r#"<ClaML version="2.0"><Class code="A00" kind="category"/></ClaML>"#;
        /// let doc = ClamlDocument::from_str(xml).unwrap();
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// let entry = index.get(&Icd10Code::from_str("A00").unwrap()).unwrap();
        /// assert_eq!(entry.title(), ""); // no preferred rubric in this fixture
        /// ```
        pub fn title(&self) -> &str {
            &self.title
        }

        /// The raw ClaML `kind` attribute, e.g. `"category"`.
        ///
        /// Captured verbatim, as `who_fic_claml::Class::kind` does — the set
        /// of kinds is open-ended (defined by the source file's own
        /// `ClassKinds` declaration), not a fixed enum.
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use who_fic_icd::icd10::Icd10Code;
        /// use std::str::FromStr;
        ///
        /// let xml = r#"<ClaML version="2.0"><Class code="A00" kind="category"/></ClaML>"#;
        /// let doc = ClamlDocument::from_str(xml).unwrap();
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// let entry = index.get(&Icd10Code::from_str("A00").unwrap()).unwrap();
        /// assert_eq!(entry.kind(), "category");
        /// ```
        pub fn kind(&self) -> &str {
            &self.kind
        }
    }

    /// A read-only, in-memory lookup from [`Icd10Code`] to its ClaML title
    /// and class metadata, built once from a user-supplied
    /// [`ClamlDocument`].
    ///
    /// This is not a live query against WHO's API — it indexes whatever
    /// `Class` elements were present in the document passed to
    /// [`Icd10ClamlIndex::from_document`].
    ///
    /// ```
    /// use who_fic_claml::ClamlDocument;
    /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
    /// use who_fic_icd::icd10::Icd10Code;
    /// use std::str::FromStr;
    ///
    /// let xml = r#"
    /// <ClaML version="2.0">
    ///   <Class code="A00-A09" kind="block">
    ///     <Rubric kind="preferred"><Label xml:lang="en">Intestinal infectious diseases</Label></Rubric>
    ///   </Class>
    ///   <Class code="A00" kind="category">
    ///     <Rubric kind="preferred"><Label xml:lang="en">Cholera</Label></Rubric>
    ///   </Class>
    /// </ClaML>
    /// "#;
    /// let doc = ClamlDocument::from_str(xml).unwrap();
    /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
    ///
    /// // The block class ("A00-A09") is not ICD-10-code-shaped and is
    /// // silently skipped; only the category class is indexed.
    /// assert_eq!(index.len(), 1);
    /// assert_eq!(index.title(&Icd10Code::from_str("A00").unwrap()), Some("Cholera"));
    /// ```
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Icd10ClamlIndex {
        entries: HashMap<Icd10Code, Icd10ClassEntry>,
    }

    impl Icd10ClamlIndex {
        /// Builds an index from a parsed ClaML document.
        ///
        /// Iterates [`ClamlDocument::classes`]; for each `Class`, attempts
        /// to parse its `code` as an [`Icd10Code`]. Classes whose code
        /// does not parse this way — chapter and block classes, whose
        /// codes are ranges like `"A00-A09"` — are expected and are
        /// silently skipped, not reported as an error: this method only
        /// fails for genuine structural problems (see
        /// [`Icd10ClamlError`]), which in the current implementation never
        /// occur, but the `Result` return type is kept for
        /// forward-compatibility with a possible future stricter-validation
        /// mode.
        ///
        /// See the type-level example above.
        pub fn from_document(doc: &ClamlDocument) -> Result<Self, Icd10ClamlError> {
            let mut entries = HashMap::new();
            for class in doc.classes() {
                let code = match Icd10Code::from_str(class.code()) {
                    Ok(code) => code,
                    // Not ICD-10-shaped (e.g. a chapter/block range like
                    // "A00-A09") — expected, not an error.
                    Err(_) => continue,
                };
                let title = class.preferred_label("en").unwrap_or("").to_string();
                let kind = class.kind().to_string();
                entries.insert(code.clone(), Icd10ClassEntry { code, title, kind });
            }
            Ok(Self { entries })
        }

        /// The title of the given code, if it is in the index.
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use who_fic_icd::icd10::Icd10Code;
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
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// assert_eq!(index.title(&Icd10Code::from_str("A00").unwrap()), Some("Cholera"));
        /// assert_eq!(index.title(&Icd10Code::from_str("Z99").unwrap()), None);
        /// ```
        pub fn title(&self, code: &Icd10Code) -> Option<&str> {
            self.entries.get(code).map(|entry| entry.title.as_str())
        }

        /// The full entry for the given code, if it is in the index.
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use who_fic_icd::icd10::Icd10Code;
        /// use std::str::FromStr;
        ///
        /// let xml = r#"<ClaML version="2.0"><Class code="A00" kind="category"/></ClaML>"#;
        /// let doc = ClamlDocument::from_str(xml).unwrap();
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// assert!(index.get(&Icd10Code::from_str("A00").unwrap()).is_some());
        /// assert!(index.get(&Icd10Code::from_str("Z99").unwrap()).is_none());
        /// ```
        pub fn get(&self, code: &Icd10Code) -> Option<&Icd10ClassEntry> {
            self.entries.get(code)
        }

        /// Iterates every indexed entry, in unspecified order.
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use std::str::FromStr;
        ///
        /// let xml = r#"<ClaML version="2.0"><Class code="A00" kind="category"/></ClaML>"#;
        /// let doc = ClamlDocument::from_str(xml).unwrap();
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// assert_eq!(index.iter().count(), 1);
        /// ```
        pub fn iter(&self) -> impl Iterator<Item = &Icd10ClassEntry> {
            self.entries.values()
        }

        /// The number of indexed entries.
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use std::str::FromStr;
        ///
        /// let doc = ClamlDocument::from_str(r#"<ClaML version="2.0"/>"#).unwrap();
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// assert_eq!(index.len(), 0);
        /// ```
        pub fn len(&self) -> usize {
            self.entries.len()
        }

        /// Returns `true` if the index has no entries.
        ///
        /// ```
        /// use who_fic_claml::ClamlDocument;
        /// use who_fic_icd::icd10::claml::Icd10ClamlIndex;
        /// use std::str::FromStr;
        ///
        /// let doc = ClamlDocument::from_str(r#"<ClaML version="2.0"/>"#).unwrap();
        /// let index = Icd10ClamlIndex::from_document(&doc).unwrap();
        /// assert!(index.is_empty());
        /// ```
        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }
    }

    /// An error building an [`Icd10ClamlIndex`].
    ///
    /// In the current implementation, [`Icd10ClamlIndex::from_document`]
    /// never actually returns `Err`: a `Class` whose `code` does not parse
    /// as an [`Icd10Code`] (chapter/block ranges like `"A00-A09"`) is
    /// expected and is silently skipped rather than erroring. This type is
    /// kept so that behavior can be extended without a breaking API
    /// change — for example a possible future stricter-validation
    /// constructor that treats an unparseable code as an error instead of
    /// skipping it, which would report [`Icd10ClamlError::InvalidCode`].
    ///
    /// `#[non_exhaustive]` so new variants can be added without a breaking
    /// change.
    ///
    /// ```
    /// use who_fic_icd::icd10::claml::Icd10ClamlError;
    ///
    /// fn assert_error_trait<E: std::error::Error>() {}
    /// assert_error_trait::<Icd10ClamlError>();
    /// ```
    #[non_exhaustive]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Icd10ClamlError {
        /// A `Class` element's `code` did not parse as an [`Icd10Code`].
        ///
        /// Not produced by [`Icd10ClamlIndex::from_document`] today (such
        /// classes are silently skipped); reserved for a possible future
        /// stricter-validation mode.
        InvalidCode {
            /// The class code that failed to parse.
            code: String,
            /// The underlying parse error.
            source: Icd10ParseError,
        },
    }

    impl fmt::Display for Icd10ClamlError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidCode { code, source } => {
                    write!(
                        f,
                        "ClaML class code {code:?} is not a valid ICD-10 code: {source}"
                    )
                }
            }
        }
    }

    impl std::error::Error for Icd10ClamlError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::InvalidCode { source, .. } => Some(source),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Mixes a `kind="category"` class with a valid ICD-10 code with a
        /// `kind="block"` class whose code is a range, not a single ICD-10
        /// code — the normal shape of a real ClaML document.
        const FIXTURE: &str = r#"
<ClaML version="2.0">
  <Class code="A00-A09" kind="block">
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
  </Class>
  <Class code="A01" kind="category">
    <SuperClass code="A00-A09"/>
    <Rubric kind="preferred">
      <Label xml:lang="en">Typhoid and paratyphoid fevers</Label>
    </Rubric>
  </Class>
</ClaML>
"#;

        fn fixture_index() -> Icd10ClamlIndex {
            let doc = ClamlDocument::from_str(FIXTURE).expect("fixture should parse");
            Icd10ClamlIndex::from_document(&doc).expect("index should build")
        }

        #[test]
        fn title_works_for_indexed_category_code() {
            let index = fixture_index();
            let a00 = Icd10Code::from_str("A00").unwrap();
            assert_eq!(index.title(&a00), Some("Cholera"));
            assert_eq!(index.get(&a00).unwrap().kind(), "category");
        }

        #[test]
        fn title_returns_none_for_code_never_in_fixture() {
            let index = fixture_index();
            let z99 = Icd10Code::from_str("Z99").unwrap();
            assert_eq!(index.title(&z99), None);
            assert!(index.get(&z99).is_none());
        }

        #[test]
        fn block_class_with_range_code_is_silently_skipped() {
            let index = fixture_index();
            // Only the 2 category classes are indexed; the block class
            // ("A00-A09") is not ICD-10-code-shaped.
            assert_eq!(index.len(), 2);
            assert!(index.iter().all(|entry| entry.code().as_str() != "A00-A09"));
        }

        #[test]
        fn missing_preferred_label_defaults_to_empty_title() {
            let xml = r#"<ClaML version="2.0"><Class code="B00" kind="category"/></ClaML>"#;
            let doc = ClamlDocument::from_str(xml).unwrap();
            let index = Icd10ClamlIndex::from_document(&doc).unwrap();
            let b00 = Icd10Code::from_str("B00").unwrap();
            assert_eq!(index.title(&b00), Some(""));
        }

        #[cfg(feature = "serde")]
        #[test]
        fn entry_serde_round_trip() {
            let index = fixture_index();
            let a00 = Icd10Code::from_str("A00").unwrap();
            let entry = index.get(&a00).unwrap();
            let json = serde_json::to_string(entry).unwrap();
            let back: Icd10ClassEntry = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_good_codes() {
        for good in ["A00", "I63.9", "U07.1", "z99", "a00.a1"] {
            assert!(
                Icd10Code::from_str(good).is_ok(),
                "expected {good:?} to parse"
            );
        }
    }

    #[test]
    fn round_trips_to_canonical_uppercase() {
        let code = Icd10Code::from_str("i63.9").unwrap();
        assert_eq!(code.as_str(), "I63.9");
        assert_eq!(code.to_string(), "I63.9");
        assert_eq!(Icd10Code::from_str(&code.to_string()).unwrap(), code);
    }

    #[test]
    fn category_and_subdivision_accessors() {
        let code = Icd10Code::from_str("I63.9").unwrap();
        assert_eq!(code.category(), "I63");
        assert_eq!(code.subdivision(), Some("9"));

        let no_sub = Icd10Code::from_str("A00").unwrap();
        assert_eq!(no_sub.category(), "A00");
        assert_eq!(no_sub.subdivision(), None);
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(Icd10Code::from_str(""), Err(Icd10ParseError::Empty));
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(
            Icd10Code::from_str("A0"),
            Err(Icd10ParseError::InvalidLength { found: 2 })
        );
    }

    #[test]
    fn rejects_bad_letter() {
        assert_eq!(
            Icd10Code::from_str("100"),
            Err(Icd10ParseError::InvalidCharacter {
                position: 0,
                found: '1'
            })
        );
    }

    #[test]
    fn rejects_bad_digit() {
        assert_eq!(
            Icd10Code::from_str("AX0"),
            Err(Icd10ParseError::InvalidCharacter {
                position: 1,
                found: 'X'
            })
        );
    }

    #[test]
    fn rejects_missing_dot() {
        assert_eq!(
            Icd10Code::from_str("A009"),
            Err(Icd10ParseError::InvalidCharacter {
                position: 3,
                found: '9'
            })
        );
    }

    #[test]
    fn rejects_empty_subdivision() {
        assert_eq!(
            Icd10Code::from_str("A00."),
            Err(Icd10ParseError::InvalidStructure {
                reason: "subdivision must have 1 to 2 characters after '.'"
            })
        );
    }

    #[test]
    fn rejects_dagger_and_asterisk_markers() {
        assert!(matches!(
            Icd10Code::from_str("A00\u{2020}"),
            Err(Icd10ParseError::InvalidCharacter { .. })
        ));
        assert!(matches!(
            Icd10Code::from_str("A00*"),
            Err(Icd10ParseError::InvalidCharacter { .. })
        ));
    }

    #[test]
    fn rejects_7_character_national_extension_codes() {
        // ICD-10-CM style 7-character codes are explicitly out of scope.
        assert_eq!(
            Icd10Code::from_str("S72001A"),
            Err(Icd10ParseError::InvalidLength { found: 7 })
        );
        assert_eq!(
            Icd10Code::from_str("S72.001A"),
            Err(Icd10ParseError::InvalidLength { found: 8 })
        );
    }

    #[test]
    fn rejects_icd11_shaped_code() {
        // First character is a digit, never valid for ICD-10 (category must
        // start with a letter).
        assert!(matches!(
            Icd10Code::from_str("8B20"),
            Err(Icd10ParseError::InvalidCharacter { position: 0, .. })
        ));
    }

    #[test]
    fn chapter_boundaries() {
        use Icd10Chapter::*;
        let cases: &[(&str, Option<Icd10Chapter>)] = &[
            ("A00", Some(I)),
            ("B99", Some(I)),
            ("C00", Some(II)),
            ("D48", Some(II)),
            ("D49", None),
            ("D50", Some(III)),
            ("D89", Some(III)),
            ("E00", Some(IV)),
            ("E90", Some(IV)),
            ("E91", None),
            ("F00", Some(V)),
            ("F99", Some(V)),
            ("G00", Some(VI)),
            ("G99", Some(VI)),
            ("H00", Some(VII)),
            ("H59", Some(VII)),
            ("H60", Some(VIII)),
            ("H95", Some(VIII)),
            ("I00", Some(IX)),
            ("I99", Some(IX)),
            ("J00", Some(X)),
            ("J99", Some(X)),
            ("K00", Some(XI)),
            ("K93", Some(XI)),
            ("K94", None),
            ("L00", Some(XII)),
            ("L99", Some(XII)),
            ("M00", Some(XIII)),
            ("M99", Some(XIII)),
            ("N00", Some(XIV)),
            ("N99", Some(XIV)),
            ("O00", Some(XV)),
            ("O99", Some(XV)),
            ("P00", Some(XVI)),
            ("P96", Some(XVI)),
            ("P97", None),
            ("Q00", Some(XVII)),
            ("Q99", Some(XVII)),
            ("R00", Some(XVIII)),
            ("R99", Some(XVIII)),
            ("S00", Some(XIX)),
            ("T98", Some(XIX)),
            ("T99", None),
            ("U00", Some(XXII)),
            ("U99", Some(XXII)),
            ("V00", None),
            ("V01", Some(XX)),
            ("Y98", Some(XX)),
            ("Y99", None),
            ("Z00", Some(XXI)),
            ("Z99", Some(XXI)),
        ];
        for (category, expected) in cases {
            assert_eq!(
                Icd10Chapter::from_category(category),
                *expected,
                "category {category}"
            );
        }
    }

    #[test]
    fn code_chapter_accessor_matches_category_lookup() {
        let code = Icd10Code::from_str("I63.9").unwrap();
        assert_eq!(code.chapter(), Some(Icd10Chapter::IX));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        let code = Icd10Code::from_str("I63.9").unwrap();
        let json = serde_json_like_roundtrip(&code);
        assert_eq!(json, code);
    }

    #[cfg(feature = "serde")]
    fn serde_json_like_roundtrip(code: &Icd10Code) -> Icd10Code {
        let s = serde_json::to_string(code).unwrap();
        assert_eq!(s, "\"I63.9\"");
        serde_json::from_str(&s).unwrap()
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_invalid_string() {
        let err = serde_json::from_str::<Icd10Code>("\"not a code\"");
        assert!(err.is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_chapter_round_trip() {
        let chapter = Icd10Chapter::IX;
        let s = serde_json::to_string(&chapter).unwrap();
        let back: Icd10Chapter = serde_json::from_str(&s).unwrap();
        assert_eq!(back, chapter);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn valid_icd10_code_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            proptest::string::string_regex("[A-Za-z][0-9]{2}").unwrap(),
            proptest::string::string_regex("[A-Za-z][0-9]{2}\\.[A-Za-z0-9]").unwrap(),
            proptest::string::string_regex("[A-Za-z][0-9]{2}\\.[A-Za-z0-9]{2}").unwrap(),
        ]
    }

    proptest! {
        #[test]
        fn round_trip_parse_format_parse(s in valid_icd10_code_strategy()) {
            let parsed = Icd10Code::from_str(&s).expect("valid-shaped string should parse");
            let formatted = parsed.to_string();
            let reparsed = Icd10Code::from_str(&formatted).expect("canonical form should reparse");
            prop_assert_eq!(parsed, reparsed);
        }

        #[test]
        fn arbitrary_strings_never_panic(s in ".*") {
            let _ = Icd10Code::from_str(&s);
        }
    }
}
