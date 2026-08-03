//! ICD-11: the Eleventh Revision of the International Classification of
//! Diseases, in effect since 2022.
//!
//! ICD-11 distinguishes the *Foundation* (a semantic network of entities
//! with URIs) from *linearizations*, chiefly the MMS (Mortality and
//! Morbidity Statistics), which is where codes live. This module models
//! MMS codes. Foundation URIs/entity IDs are out of scope for this crate.
//!
//! # Code syntax
//!
//! ```text
//! stem        = chapterchar letter alnum alnum [ "." 1*2alnum ]
//! chapterchar = digit / letter                  ; identifies the chapter
//! alnum       = digit / letter
//! letter      = A–Z excluding "I" and "O"        ; excluded to avoid 1/0 confusion
//! ```
//!
//! - Stem codes are 4 characters before any subdivision, e.g. `8B20`
//!   (stroke), `CA40` (pneumonia), with subdivisions like `CA40.0`.
//! - The letters `I` and `O` never appear anywhere in an ICD-11 code.
//! - The second character is always a letter — this distinguishes ICD-11
//!   codes from ICD-10 codes (`letter digit digit`) at a glance.
//! - Codes beginning with `X` are [`ExtensionCode`]s, used only in
//!   postcoordination ([`Cluster`]); [`Icd11Code`] rejects them.
//!
//! Parsing validates syntax and structure only, not whether WHO has
//! actually assigned a given code.

use std::fmt;
use std::str::FromStr;

/// An ICD-11 MMS stem code, e.g. `8B20` or `CA40.0`.
///
/// Wraps a validated, canonical (uppercase) string form. The only way to
/// construct one is by parsing (`FromStr`/`TryFrom<&str>`).
///
/// Codes beginning with `X` are extension codes, not stem codes; use
/// [`ExtensionCode`] for those — [`Icd11Code::from_str`] rejects them.
///
/// ```
/// use who_fic_icd::icd11::Icd11Code;
///
/// let code: Icd11Code = "ca40.0".parse().unwrap();
/// assert_eq!(code.as_str(), "CA40.0");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Icd11Code(String);

impl Icd11Code {
    /// Returns the canonical (uppercase) string form of the code.
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Code;
    ///
    /// let code: Icd11Code = "8b20".parse().unwrap();
    /// assert_eq!(code.as_str(), "8B20");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the 4-character stem, e.g. `"CA40"` for `CA40.0`.
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Code;
    ///
    /// let code: Icd11Code = "CA40.0".parse().unwrap();
    /// assert_eq!(code.stem(), "CA40");
    /// ```
    pub fn stem(&self) -> &str {
        &self.0[0..4]
    }

    /// Returns the subdivision (the part after the `.`), if present.
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Code;
    ///
    /// let with_sub: Icd11Code = "CA40.0".parse().unwrap();
    /// assert_eq!(with_sub.subdivision(), Some("0"));
    ///
    /// let without_sub: Icd11Code = "8B20".parse().unwrap();
    /// assert_eq!(without_sub.subdivision(), None);
    /// ```
    pub fn subdivision(&self) -> Option<&str> {
        if self.0.len() > 4 {
            Some(&self.0[5..])
        } else {
            None
        }
    }

    /// Returns `true` if the code's subdivision is `Y` ("other specified").
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Code;
    ///
    /// let code: Icd11Code = "8B20.Y".parse().unwrap();
    /// assert!(code.is_residual_other());
    /// ```
    pub fn is_residual_other(&self) -> bool {
        self.0.ends_with(".Y")
    }

    /// Returns `true` if the code's subdivision is `Z` ("unspecified").
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Code;
    ///
    /// let code: Icd11Code = "8B20.Z".parse().unwrap();
    /// assert!(code.is_residual_unspecified());
    /// ```
    pub fn is_residual_unspecified(&self) -> bool {
        self.0.ends_with(".Z")
    }

    /// Returns the chapter identified by this code's leading character, or
    /// `None` if the leading character is not (yet) assigned to any
    /// chapter.
    ///
    /// ```
    /// use who_fic_icd::icd11::{Icd11Chapter, Icd11Code};
    ///
    /// let code: Icd11Code = "8B20".parse().unwrap();
    /// assert_eq!(code.chapter(), Some(Icd11Chapter::Chapter08));
    /// ```
    pub fn chapter(&self) -> Option<Icd11Chapter> {
        Icd11Chapter::from_leading_char(self.0.as_bytes()[0] as char)
    }
}

impl fmt::Display for Icd11Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Icd11Code {
    type Err = Icd11ParseError;

    /// Parses an ICD-11 stem code. Case-insensitive; whitespace is not
    /// trimmed. Rejects `X`-prefixed input — use [`ExtensionCode`] instead.
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Code;
    /// use std::str::FromStr;
    ///
    /// assert!(Icd11Code::from_str("8B20").is_ok());
    /// assert!(Icd11Code::from_str("XK9J").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let canonical = parse_stem_shape(s)?;
        if canonical.as_bytes()[0] == b'X' {
            return Err(Icd11ParseError::InvalidStructure {
                reason: "codes beginning with 'X' are extension codes; use ExtensionCode::from_str instead",
            });
        }
        Ok(Icd11Code(canonical))
    }
}

impl TryFrom<&str> for Icd11Code {
    type Error = Icd11ParseError;

    /// ```
    /// use who_fic_icd::icd11::Icd11Code;
    ///
    /// let code = Icd11Code::try_from("1A00").unwrap();
    /// assert_eq!(code.as_str(), "1A00");
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Icd11Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Icd11Code {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// An ICD-11 extension code, e.g. `XK9J`.
///
/// Extension codes (severity, anatomy, temporality, etc.) are used only in
/// postcoordination — attached to a stem code via `&` in a [`Cluster`] —
/// never alone. They are a separate type from [`Icd11Code`] so that
/// stem-only APIs are type-enforced: an `ExtensionCode` cannot be passed
/// where an `Icd11Code` is expected, and vice versa.
///
/// ```
/// use who_fic_icd::icd11::ExtensionCode;
///
/// let ext: ExtensionCode = "xk9j".parse().unwrap();
/// assert_eq!(ext.as_str(), "XK9J");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExtensionCode(String);

impl ExtensionCode {
    /// Returns the canonical (uppercase) string form of the extension code.
    ///
    /// ```
    /// use who_fic_icd::icd11::ExtensionCode;
    ///
    /// let ext: ExtensionCode = "XK9J".parse().unwrap();
    /// assert_eq!(ext.as_str(), "XK9J");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the 4-character stem of the extension code (including the
    /// leading `X`), e.g. `"XK9J"`.
    ///
    /// ```
    /// use who_fic_icd::icd11::ExtensionCode;
    ///
    /// let ext: ExtensionCode = "XK9J".parse().unwrap();
    /// assert_eq!(ext.stem(), "XK9J");
    /// ```
    pub fn stem(&self) -> &str {
        &self.0[0..4]
    }

    /// Returns the subdivision (the part after the `.`), if present.
    ///
    /// ```
    /// use who_fic_icd::icd11::ExtensionCode;
    ///
    /// let ext: ExtensionCode = "XK9J.1".parse().unwrap();
    /// assert_eq!(ext.subdivision(), Some("1"));
    /// ```
    pub fn subdivision(&self) -> Option<&str> {
        if self.0.len() > 4 {
            Some(&self.0[5..])
        } else {
            None
        }
    }
}

impl fmt::Display for ExtensionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ExtensionCode {
    type Err = Icd11ParseError;

    /// Parses an ICD-11 extension code. Case-insensitive; whitespace is not
    /// trimmed. Requires an `X` prefix — use [`Icd11Code`] for stem codes.
    ///
    /// ```
    /// use who_fic_icd::icd11::ExtensionCode;
    /// use std::str::FromStr;
    ///
    /// assert!(ExtensionCode::from_str("XK9J").is_ok());
    /// assert!(ExtensionCode::from_str("8B20").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let canonical = parse_stem_shape(s)?;
        if canonical.as_bytes()[0] != b'X' {
            return Err(Icd11ParseError::InvalidStructure {
                reason: "extension codes must begin with 'X'; use Icd11Code::from_str for stem codes",
            });
        }
        Ok(ExtensionCode(canonical))
    }
}

impl TryFrom<&str> for ExtensionCode {
    type Error = Icd11ParseError;

    /// ```
    /// use who_fic_icd::icd11::ExtensionCode;
    ///
    /// let ext = ExtensionCode::try_from("XK9J").unwrap();
    /// assert_eq!(ext.as_str(), "XK9J");
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ExtensionCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ExtensionCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Parses the shared `chapterchar letter alnum alnum [ "." 1*2alnum ]`
/// shape used by both [`Icd11Code`] and [`ExtensionCode`], without
/// enforcing whether the leading character is or isn't `X`. Returns the
/// canonical (uppercase) string form.
fn parse_stem_shape(s: &str) -> Result<String, Icd11ParseError> {
    if s.is_empty() {
        return Err(Icd11ParseError::Empty);
    }
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len < 4 {
        return Err(Icd11ParseError::InvalidLength { found: len });
    }

    let mut canonical = String::with_capacity(len);

    // Position 0: chapterchar = digit or letter (excluding I/O).
    check_alnum_no_io(chars[0], 0)?;
    canonical.push(chars[0].to_ascii_uppercase());

    // Position 1: always a letter (excluding I/O) — distinguishes ICD-11
    // from ICD-10 at a glance.
    check_letter_no_io(chars[1], 1)?;
    canonical.push(chars[1].to_ascii_uppercase());

    // Positions 2-3: alnum (excluding I/O).
    for (pos, c) in chars.iter().enumerate().take(4).skip(2) {
        check_alnum_no_io(*c, pos)?;
        canonical.push(c.to_ascii_uppercase());
    }

    if len == 4 {
        return Ok(canonical);
    }
    if len > 7 {
        return Err(Icd11ParseError::InvalidLength { found: len });
    }
    if chars[4] != '.' {
        return Err(Icd11ParseError::InvalidCharacter {
            position: 4,
            found: chars[4],
        });
    }
    let subdivision = &chars[5..];
    if subdivision.is_empty() {
        return Err(Icd11ParseError::InvalidStructure {
            reason: "subdivision must have 1 to 2 characters after '.'",
        });
    }
    canonical.push('.');
    for (offset, c) in subdivision.iter().enumerate() {
        check_alnum_no_io(*c, 5 + offset)?;
        canonical.push(c.to_ascii_uppercase());
    }
    Ok(canonical)
}

fn check_alnum_no_io(c: char, position: usize) -> Result<(), Icd11ParseError> {
    if !c.is_ascii_alphanumeric() {
        return Err(Icd11ParseError::InvalidCharacter { position, found: c });
    }
    check_not_i_or_o(c, position)
}

fn check_letter_no_io(c: char, position: usize) -> Result<(), Icd11ParseError> {
    if !c.is_ascii_alphabetic() {
        return Err(Icd11ParseError::InvalidCharacter { position, found: c });
    }
    check_not_i_or_o(c, position)
}

fn check_not_i_or_o(c: char, position: usize) -> Result<(), Icd11ParseError> {
    match c.to_ascii_uppercase() {
        'I' | 'O' => Err(Icd11ParseError::InvalidCharacter { position, found: c }),
        _ => Ok(()),
    }
}

/// The chapters of the ICD-11 MMS linearization: chapters 01–26, plus `V`
/// (supplementary section for functioning assessment) and `X` (extension
/// codes).
///
/// The leading character of a code always identifies its chapter — see the
/// [ICD-11 MMS reference guide](https://icdcdn.who.int/icd11referenceguide/en/html/index.html).
/// This is WHO's official chapter-to-leading-character table (a structural
/// fact, not copyrighted classification content):
///
/// | Chapter | Char | Chapter | Char | Chapter | Char |
/// |---------|------|---------|------|---------|------|
/// | 01      | `1`  | 10      | `A`  | 19      | `K`  |
/// | 02      | `2`  | 11      | `B`  | 20      | `L`  |
/// | 03      | `3`  | 12      | `C`  | 21      | `M`  |
/// | 04      | `4`  | 13      | `D`  | 22      | `N`  |
/// | 05      | `5`  | 14      | `E`  | 23      | `P`  |
/// | 06      | `6`  | 15      | `F`  | 24      | `Q`  |
/// | 07      | `7`  | 16      | `G`  | 25      | `R`  |
/// | 08      | `8`  | 17      | `H`  | 26      | `S`  |
/// | 09      | `9`  | 18      | `J`  | V       | `V`  |
/// |         |      |         |      | X       | `X`  |
///
/// Note chapters 18 and 23 use `J` and `P` respectively, skipping `I` and
/// `O` (never valid in ICD-11 codes). The leading characters `0`, `T`, `U`,
/// `W`, `Y`, `Z` are not (yet) assigned to any chapter;
/// [`Icd11Code::chapter`] returns `None` for those.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Icd11Chapter {
    /// 01: Certain infectious or parasitic diseases (leading char `1`).
    Chapter01,
    /// 02: Neoplasms (leading char `2`).
    Chapter02,
    /// 03: Diseases of the blood or blood-forming organs (leading char `3`).
    Chapter03,
    /// 04: Diseases of the immune system (leading char `4`).
    Chapter04,
    /// 05: Endocrine, nutritional or metabolic diseases (leading char `5`).
    Chapter05,
    /// 06: Mental, behavioural or neurodevelopmental disorders (leading
    /// char `6`).
    Chapter06,
    /// 07: Sleep-wake disorders (leading char `7`).
    Chapter07,
    /// 08: Diseases of the nervous system (leading char `8`).
    Chapter08,
    /// 09: Diseases of the visual system (leading char `9`).
    Chapter09,
    /// 10: Diseases of the ear or mastoid process (leading char `A`).
    Chapter10,
    /// 11: Diseases of the circulatory system (leading char `B`).
    Chapter11,
    /// 12: Diseases of the respiratory system (leading char `C`).
    Chapter12,
    /// 13: Diseases of the digestive system (leading char `D`).
    Chapter13,
    /// 14: Diseases of the skin (leading char `E`).
    Chapter14,
    /// 15: Diseases of the musculoskeletal system or connective tissue
    /// (leading char `F`).
    Chapter15,
    /// 16: Diseases of the genitourinary system (leading char `G`).
    Chapter16,
    /// 17: Conditions related to sexual health (leading char `H`).
    Chapter17,
    /// 18: Pregnancy, childbirth or the puerperium (leading char `J`).
    Chapter18,
    /// 19: Certain conditions originating in the perinatal period (leading
    /// char `K`).
    Chapter19,
    /// 20: Developmental anomalies (leading char `L`).
    Chapter20,
    /// 21: Symptoms, signs or clinical findings, not elsewhere classified
    /// (leading char `M`).
    Chapter21,
    /// 22: Injury, poisoning or certain other consequences of external
    /// causes (leading char `N`).
    Chapter22,
    /// 23: External causes of morbidity or mortality (leading char `P`).
    Chapter23,
    /// 24: Factors influencing health status or contact with health
    /// services (leading char `Q`).
    Chapter24,
    /// 25: Codes for special purposes (leading char `R`).
    Chapter25,
    /// 26: Supplementary Chapter Traditional Medicine Conditions — Module I
    /// (leading char `S`).
    Chapter26,
    /// V: Supplementary section for functioning assessment (leading char
    /// `V`).
    ChapterV,
    /// X: Extension codes, used only in postcoordination (leading char
    /// `X`).
    ChapterX,
}

impl Icd11Chapter {
    /// Returns the chapter identified by the given leading character
    /// (case-insensitive), or `None` if it is not assigned to any chapter.
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Chapter;
    ///
    /// assert_eq!(Icd11Chapter::from_leading_char('8'), Some(Icd11Chapter::Chapter08));
    /// assert_eq!(Icd11Chapter::from_leading_char('x'), Some(Icd11Chapter::ChapterX));
    /// assert_eq!(Icd11Chapter::from_leading_char('T'), None); // unassigned
    /// ```
    pub fn from_leading_char(c: char) -> Option<Self> {
        use Icd11Chapter::*;
        match c.to_ascii_uppercase() {
            '1' => Some(Chapter01),
            '2' => Some(Chapter02),
            '3' => Some(Chapter03),
            '4' => Some(Chapter04),
            '5' => Some(Chapter05),
            '6' => Some(Chapter06),
            '7' => Some(Chapter07),
            '8' => Some(Chapter08),
            '9' => Some(Chapter09),
            'A' => Some(Chapter10),
            'B' => Some(Chapter11),
            'C' => Some(Chapter12),
            'D' => Some(Chapter13),
            'E' => Some(Chapter14),
            'F' => Some(Chapter15),
            'G' => Some(Chapter16),
            'H' => Some(Chapter17),
            'J' => Some(Chapter18),
            'K' => Some(Chapter19),
            'L' => Some(Chapter20),
            'M' => Some(Chapter21),
            'N' => Some(Chapter22),
            'P' => Some(Chapter23),
            'Q' => Some(Chapter24),
            'R' => Some(Chapter25),
            'S' => Some(Chapter26),
            'V' => Some(ChapterV),
            'X' => Some(ChapterX),
            _ => None,
        }
    }

    /// Returns the canonical leading character for this chapter.
    ///
    /// ```
    /// use who_fic_icd::icd11::Icd11Chapter;
    ///
    /// assert_eq!(Icd11Chapter::Chapter01.leading_char(), '1');
    /// assert_eq!(Icd11Chapter::ChapterX.leading_char(), 'X');
    /// ```
    pub fn leading_char(&self) -> char {
        use Icd11Chapter::*;
        match self {
            Chapter01 => '1',
            Chapter02 => '2',
            Chapter03 => '3',
            Chapter04 => '4',
            Chapter05 => '5',
            Chapter06 => '6',
            Chapter07 => '7',
            Chapter08 => '8',
            Chapter09 => '9',
            Chapter10 => 'A',
            Chapter11 => 'B',
            Chapter12 => 'C',
            Chapter13 => 'D',
            Chapter14 => 'E',
            Chapter15 => 'F',
            Chapter16 => 'G',
            Chapter17 => 'H',
            Chapter18 => 'J',
            Chapter19 => 'K',
            Chapter20 => 'L',
            Chapter21 => 'M',
            Chapter22 => 'N',
            Chapter23 => 'P',
            Chapter24 => 'Q',
            Chapter25 => 'R',
            Chapter26 => 'S',
            ChapterV => 'V',
            ChapterX => 'X',
        }
    }
}

/// An error parsing an [`Icd11Code`], [`ExtensionCode`], or [`Cluster`].
///
/// Follows the shared error shape used across `who-fic` crates (see
/// `specs/architecture.md`): empty input, invalid length, an invalid
/// character at a known position, or a structural problem that isn't
/// reducible to a single bad character or length.
///
/// ```
/// use who_fic_icd::icd11::{Icd11Code, Icd11ParseError};
/// use std::str::FromStr;
///
/// assert_eq!(Icd11Code::from_str(""), Err(Icd11ParseError::Empty));
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Icd11ParseError {
    /// The input was empty.
    Empty,
    /// The input's overall length is not valid for any ICD-11 code shape.
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

impl fmt::Display for Icd11ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "ICD-11 code is empty"),
            Self::InvalidLength { found } => {
                write!(f, "invalid ICD-11 code length: {found}")
            }
            Self::InvalidCharacter { position, found } => {
                write!(f, "invalid character {found:?} at position {position}")
            }
            Self::InvalidStructure { reason } => {
                write!(f, "invalid ICD-11 code structure: {reason}")
            }
        }
    }
}

impl std::error::Error for Icd11ParseError {}

/// One stem code together with any extension codes attached to it via `&`
/// in a postcoordination [`Cluster`].
///
/// ```
/// use who_fic_icd::icd11::Cluster;
/// use std::str::FromStr;
///
/// let cluster = Cluster::from_str("8B20&XK9J").unwrap();
/// let group = &cluster.groups()[0];
/// assert_eq!(group.stem().as_str(), "8B20");
/// assert_eq!(group.extensions()[0].as_str(), "XK9J");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClusterStem {
    stem: Icd11Code,
    extensions: Vec<ExtensionCode>,
}

impl ClusterStem {
    /// Returns the stem code of this group.
    ///
    /// ```
    /// use who_fic_icd::icd11::Cluster;
    /// use std::str::FromStr;
    ///
    /// let cluster = Cluster::from_str("8B20&XK9J").unwrap();
    /// assert_eq!(cluster.groups()[0].stem().as_str(), "8B20");
    /// ```
    pub fn stem(&self) -> &Icd11Code {
        &self.stem
    }

    /// Returns the extension codes attached to this group's stem, in the
    /// order they appeared in the cluster string.
    ///
    /// ```
    /// use who_fic_icd::icd11::Cluster;
    /// use std::str::FromStr;
    ///
    /// let cluster = Cluster::from_str("8B20&XK9J").unwrap();
    /// assert_eq!(cluster.groups()[0].extensions().len(), 1);
    /// ```
    pub fn extensions(&self) -> &[ExtensionCode] {
        &self.extensions
    }
}

impl fmt::Display for ClusterStem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stem)?;
        for extension in &self.extensions {
            write!(f, "&{extension}")?;
        }
        Ok(())
    }
}

/// A parsed ICD-11 postcoordination cluster string.
///
/// ICD-11 codes combine into clusters using two separators:
///
/// - `&` joins a stem code with one or more extension codes, e.g.
///   `8B20&XK9J`.
/// - `/` joins multiple stem groups into one clinical concept, e.g.
///   `NA07.1/8B20`.
///
/// **Representation choice** (the spec leaves this open — documented here):
/// a cluster is modeled as an ordered list of [`ClusterStem`] groups, one
/// per `/`-separated segment, each holding one required stem code and zero
/// or more `&`-attached extension codes. This directly mirrors the two
/// separators' precedence (`/` splits the whole string into groups, `&`
/// splits each group into a stem plus its extensions) and supports both
/// example forms — and their combination, e.g. `NA07.1/8B20&XK9J` — without
/// a special case.
///
/// This parses **syntax only**: whether a given extension is actually
/// permitted on a given stem requires WHO data and is out of scope.
///
/// ```
/// use who_fic_icd::icd11::Cluster;
/// use std::str::FromStr;
///
/// let cluster = Cluster::from_str("NA07.1/8B20").unwrap();
/// assert_eq!(cluster.groups().len(), 2);
/// assert_eq!(cluster.to_string(), "NA07.1/8B20");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cluster {
    groups: Vec<ClusterStem>,
}

impl Cluster {
    /// Returns the parsed `/`-separated groups, in order.
    ///
    /// ```
    /// use who_fic_icd::icd11::Cluster;
    /// use std::str::FromStr;
    ///
    /// let cluster = Cluster::from_str("8B20&XK9J").unwrap();
    /// assert_eq!(cluster.groups().len(), 1);
    /// ```
    pub fn groups(&self) -> &[ClusterStem] {
        &self.groups
    }

    /// Iterates over every stem code in the cluster, one per group.
    ///
    /// ```
    /// use who_fic_icd::icd11::Cluster;
    /// use std::str::FromStr;
    ///
    /// let cluster = Cluster::from_str("NA07.1/8B20").unwrap();
    /// let stems: Vec<&str> = cluster.stems().map(|s| s.as_str()).collect();
    /// assert_eq!(stems, vec!["NA07.1", "8B20"]);
    /// ```
    pub fn stems(&self) -> impl Iterator<Item = &Icd11Code> + '_ {
        self.groups.iter().map(|group| &group.stem)
    }

    /// Iterates over every extension code in the cluster, across all
    /// groups, in order.
    ///
    /// ```
    /// use who_fic_icd::icd11::Cluster;
    /// use std::str::FromStr;
    ///
    /// let cluster = Cluster::from_str("8B20&XK9J").unwrap();
    /// let extensions: Vec<&str> = cluster.extensions().map(|e| e.as_str()).collect();
    /// assert_eq!(extensions, vec!["XK9J"]);
    /// ```
    pub fn extensions(&self) -> impl Iterator<Item = &ExtensionCode> + '_ {
        self.groups.iter().flat_map(|group| group.extensions.iter())
    }
}

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, group) in self.groups.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            write!(f, "{group}")?;
        }
        Ok(())
    }
}

impl FromStr for Cluster {
    type Err = Icd11ParseError;

    /// Parses a postcoordination cluster string.
    ///
    /// ```
    /// use who_fic_icd::icd11::Cluster;
    /// use std::str::FromStr;
    ///
    /// assert!(Cluster::from_str("8B20&XK9J").is_ok());
    /// assert!(Cluster::from_str("NA07.1/8B20").is_ok());
    /// assert!(Cluster::from_str("").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Icd11ParseError::Empty);
        }
        let mut groups = Vec::new();
        for segment in s.split('/') {
            if segment.is_empty() {
                return Err(Icd11ParseError::InvalidStructure {
                    reason: "cluster segment between '/' separators must not be empty",
                });
            }
            let mut parts = segment.split('&');
            // `split` on a non-empty string always yields at least one item.
            let stem_str = parts.next().unwrap();
            let stem = Icd11Code::from_str(stem_str)?;
            let mut extensions = Vec::new();
            for extension_str in parts {
                extensions.push(ExtensionCode::from_str(extension_str)?);
            }
            groups.push(ClusterStem { stem, extensions });
        }
        Ok(Cluster { groups })
    }
}

impl TryFrom<&str> for Cluster {
    type Error = Icd11ParseError;

    /// ```
    /// use who_fic_icd::icd11::Cluster;
    ///
    /// let cluster = Cluster::try_from("8B20&XK9J").unwrap();
    /// assert_eq!(cluster.groups().len(), 1);
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_good_stem_codes() {
        for good in ["8B20", "CA40.0", "1A00", "8b20", "bb1a.9k"] {
            assert!(
                Icd11Code::from_str(good).is_ok(),
                "expected {good:?} to parse"
            );
        }
    }

    #[test]
    fn accepts_known_good_extension_codes() {
        for good in ["XK9J", "xk9j", "XA00.6"] {
            assert!(
                ExtensionCode::from_str(good).is_ok(),
                "expected {good:?} to parse"
            );
        }
    }

    #[test]
    fn round_trips_to_canonical_uppercase() {
        let code = Icd11Code::from_str("ca40.0").unwrap();
        assert_eq!(code.as_str(), "CA40.0");
        assert_eq!(code.to_string(), "CA40.0");
        assert_eq!(Icd11Code::from_str(&code.to_string()).unwrap(), code);
    }

    #[test]
    fn stem_and_subdivision_accessors() {
        let code = Icd11Code::from_str("CA40.0").unwrap();
        assert_eq!(code.stem(), "CA40");
        assert_eq!(code.subdivision(), Some("0"));

        let no_sub = Icd11Code::from_str("8B20").unwrap();
        assert_eq!(no_sub.stem(), "8B20");
        assert_eq!(no_sub.subdivision(), None);
    }

    #[test]
    fn residual_flags() {
        let other: Icd11Code = "8B20.Y".parse().unwrap();
        assert!(other.is_residual_other());
        assert!(!other.is_residual_unspecified());

        let unspecified: Icd11Code = "8B20.Z".parse().unwrap();
        assert!(unspecified.is_residual_unspecified());
        assert!(!unspecified.is_residual_other());

        let neither: Icd11Code = "8B20.0".parse().unwrap();
        assert!(!neither.is_residual_other());
        assert!(!neither.is_residual_unspecified());
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(Icd11Code::from_str(""), Err(Icd11ParseError::Empty));
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(
            Icd11Code::from_str("8B2"),
            Err(Icd11ParseError::InvalidLength { found: 3 })
        );
    }

    #[test]
    fn rejects_icd10_shaped_code() {
        // Second character is a digit, never valid for ICD-11.
        assert!(matches!(
            Icd11Code::from_str("A00"),
            Err(Icd11ParseError::InvalidLength { .. })
        ));
        assert!(matches!(
            Icd11Code::from_str("A63.9"),
            Err(Icd11ParseError::InvalidCharacter { position: 1, .. })
        ));
    }

    #[test]
    fn rejects_letter_i_and_o_anywhere() {
        for bad in ["IB20", "8I20", "8BI0", "8B2I", "OB20", "8O20"] {
            assert!(
                matches!(
                    Icd11Code::from_str(bad),
                    Err(Icd11ParseError::InvalidCharacter { .. })
                ),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn rejects_digit_in_second_position() {
        assert!(matches!(
            Icd11Code::from_str("8820"),
            Err(Icd11ParseError::InvalidCharacter { position: 1, .. })
        ));
    }

    #[test]
    fn rejects_wrong_separator() {
        assert!(matches!(
            Icd11Code::from_str("8B20,0"),
            Err(Icd11ParseError::InvalidCharacter { position: 4, .. })
        ));
    }

    #[test]
    fn rejects_empty_subdivision() {
        assert_eq!(
            Icd11Code::from_str("8B20."),
            Err(Icd11ParseError::InvalidStructure {
                reason: "subdivision must have 1 to 2 characters after '.'"
            })
        );
    }

    #[test]
    fn rejects_overlong_input() {
        assert!(matches!(
            Icd11Code::from_str("8B20.ABCDEFG"),
            Err(Icd11ParseError::InvalidLength { .. })
        ));
    }

    #[test]
    fn icd11_code_rejects_x_prefixed_input() {
        assert_eq!(
            Icd11Code::from_str("XK9J"),
            Err(Icd11ParseError::InvalidStructure {
                reason: "codes beginning with 'X' are extension codes; use ExtensionCode::from_str instead",
            })
        );
    }

    #[test]
    fn extension_code_requires_x_prefix() {
        assert_eq!(
            ExtensionCode::from_str("8B20"),
            Err(Icd11ParseError::InvalidStructure {
                reason: "extension codes must begin with 'X'; use Icd11Code::from_str for stem codes",
            })
        );
    }

    #[test]
    fn chapter_leading_char_table() {
        use Icd11Chapter::*;
        let cases: &[(char, Option<Icd11Chapter>)] = &[
            ('0', None),
            ('1', Some(Chapter01)),
            ('2', Some(Chapter02)),
            ('3', Some(Chapter03)),
            ('4', Some(Chapter04)),
            ('5', Some(Chapter05)),
            ('6', Some(Chapter06)),
            ('7', Some(Chapter07)),
            ('8', Some(Chapter08)),
            ('9', Some(Chapter09)),
            ('A', Some(Chapter10)),
            ('B', Some(Chapter11)),
            ('C', Some(Chapter12)),
            ('D', Some(Chapter13)),
            ('E', Some(Chapter14)),
            ('F', Some(Chapter15)),
            ('G', Some(Chapter16)),
            ('H', Some(Chapter17)),
            ('J', Some(Chapter18)),
            ('K', Some(Chapter19)),
            ('L', Some(Chapter20)),
            ('M', Some(Chapter21)),
            ('N', Some(Chapter22)),
            ('P', Some(Chapter23)),
            ('Q', Some(Chapter24)),
            ('R', Some(Chapter25)),
            ('S', Some(Chapter26)),
            ('T', None),
            ('U', None),
            ('V', Some(ChapterV)),
            ('W', None),
            ('X', Some(ChapterX)),
            ('Y', None),
            ('Z', None),
        ];
        for (leading_char, expected) in cases {
            assert_eq!(
                Icd11Chapter::from_leading_char(*leading_char),
                *expected,
                "leading char {leading_char}"
            );
            if let Some(chapter) = expected {
                assert_eq!(chapter.leading_char(), leading_char.to_ascii_uppercase());
            }
        }
    }

    #[test]
    fn code_chapter_accessor_matches_leading_char_lookup() {
        let code = Icd11Code::from_str("8B20").unwrap();
        assert_eq!(code.chapter(), Some(Icd11Chapter::Chapter08));

        let unassigned = Icd11Code::from_str("0B20").unwrap();
        assert_eq!(unassigned.chapter(), None);
    }

    #[test]
    fn cluster_parses_ampersand_form() {
        let cluster = Cluster::from_str("8B20&XK9J").unwrap();
        assert_eq!(cluster.groups().len(), 1);
        assert_eq!(cluster.groups()[0].stem().as_str(), "8B20");
        assert_eq!(cluster.groups()[0].extensions().len(), 1);
        assert_eq!(cluster.groups()[0].extensions()[0].as_str(), "XK9J");
        assert_eq!(cluster.to_string(), "8B20&XK9J");
    }

    #[test]
    fn cluster_parses_slash_form() {
        let cluster = Cluster::from_str("NA07.1/8B20").unwrap();
        assert_eq!(cluster.groups().len(), 2);
        assert_eq!(cluster.groups()[0].stem().as_str(), "NA07.1");
        assert!(cluster.groups()[0].extensions().is_empty());
        assert_eq!(cluster.groups()[1].stem().as_str(), "8B20");
        assert_eq!(cluster.to_string(), "NA07.1/8B20");
    }

    #[test]
    fn cluster_parses_combined_form() {
        let cluster = Cluster::from_str("NA07.1/8B20&XK9J&XA00.6").unwrap();
        assert_eq!(cluster.groups().len(), 2);
        assert_eq!(cluster.groups()[1].extensions().len(), 2);
        let stems: Vec<&str> = cluster.stems().map(|s| s.as_str()).collect();
        assert_eq!(stems, vec!["NA07.1", "8B20"]);
        let extensions: Vec<&str> = cluster.extensions().map(|e| e.as_str()).collect();
        assert_eq!(extensions, vec!["XK9J", "XA00.6"]);
        assert_eq!(cluster.to_string(), "NA07.1/8B20&XK9J&XA00.6");
    }

    #[test]
    fn cluster_rejects_empty() {
        assert_eq!(Cluster::from_str(""), Err(Icd11ParseError::Empty));
    }

    #[test]
    fn cluster_rejects_empty_segment() {
        assert_eq!(
            Cluster::from_str("8B20//CA40"),
            Err(Icd11ParseError::InvalidStructure {
                reason: "cluster segment between '/' separators must not be empty",
            })
        );
    }

    #[test]
    fn cluster_propagates_stem_and_extension_errors() {
        assert!(Cluster::from_str("XK9J").is_err()); // stem position needs a stem code, not an extension
        assert!(Cluster::from_str("8B20&8B20").is_err()); // second position needs an extension code
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_icd11_code() {
        let code = Icd11Code::from_str("CA40.0").unwrap();
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"CA40.0\"");
        let back: Icd11Code = serde_json::from_str(&json).unwrap();
        assert_eq!(back, code);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_extension_code() {
        let ext = ExtensionCode::from_str("XK9J").unwrap();
        let json = serde_json::to_string(&ext).unwrap();
        assert_eq!(json, "\"XK9J\"");
        let back: ExtensionCode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ext);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_invalid_string() {
        assert!(serde_json::from_str::<Icd11Code>("\"not a code\"").is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_chapter_round_trip() {
        let chapter = Icd11Chapter::Chapter08;
        let s = serde_json::to_string(&chapter).unwrap();
        let back: Icd11Chapter = serde_json::from_str(&s).unwrap();
        assert_eq!(back, chapter);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn valid_stem_char() -> impl Strategy<Value = char> {
        prop_oneof![
            (b'0'..=b'9').prop_map(char::from),
            (b'A'..=b'Z')
                .prop_filter("no I/O", |c| *c != b'I' && *c != b'O')
                .prop_map(char::from),
        ]
    }

    fn valid_letter_no_io() -> impl Strategy<Value = char> {
        (b'A'..=b'Z')
            .prop_filter("no I/O", |c| *c != b'I' && *c != b'O')
            .prop_map(char::from)
    }

    fn valid_stem_code_strategy() -> impl Strategy<Value = String> {
        (
            valid_stem_char(),
            valid_letter_no_io(),
            valid_stem_char(),
            valid_stem_char(),
            proptest::option::of((valid_stem_char(), proptest::option::of(valid_stem_char()))),
        )
            .prop_map(|(c0, c1, c2, c3, sub)| {
                let mut s = String::new();
                s.push(c0);
                s.push(c1);
                s.push(c2);
                s.push(c3);
                if let Some((s0, s1)) = sub {
                    s.push('.');
                    s.push(s0);
                    if let Some(s1) = s1 {
                        s.push(s1);
                    }
                }
                s
            })
            // Exclude X-prefixed stems: Icd11Code::from_str rejects those by design.
            .prop_filter("not X-prefixed", |s| {
                !s.starts_with('X') && !s.starts_with('x')
            })
    }

    proptest! {
        #[test]
        fn round_trip_parse_format_parse(s in valid_stem_code_strategy()) {
            let parsed = Icd11Code::from_str(&s).expect("valid-shaped string should parse");
            let formatted = parsed.to_string();
            let reparsed = Icd11Code::from_str(&formatted).expect("canonical form should reparse");
            prop_assert_eq!(parsed, reparsed);
        }

        #[test]
        fn arbitrary_strings_never_panic_icd11_code(s in ".*") {
            let _ = Icd11Code::from_str(&s);
        }

        #[test]
        fn arbitrary_strings_never_panic_extension_code(s in ".*") {
            let _ = ExtensionCode::from_str(&s);
        }

        #[test]
        fn arbitrary_strings_never_panic_cluster(s in ".*") {
            let _ = Cluster::from_str(&s);
        }
    }
}
