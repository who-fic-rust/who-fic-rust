//! Response types returned by [`crate::IcdApiClient`].
//!
//! These mirror the JSON-LD field names documented in WHO's ICD Schema and
//! API docs (see `specs/who-fic-icd-api.md`). WHO's swagger spec does not
//! formally document response bodies, so deserialization here is
//! deliberately permissive: unknown fields are ignored, fields that might
//! legitimately be absent are `Option`, and nothing panics on an
//! unrecognized shape — at worst a response fails to parse into the
//! expected type and callers see [`crate::IcdApiError::Decode`].

use serde::Deserialize;
use std::collections::HashMap;

/// A JSON-LD language-tagged string, e.g. `{"@language": "en", "@value":
/// "Cholera"}` — the shape WHO uses for `Entity::title` and
/// `Entity::definition`.
///
/// ```
/// use who_fic_icd_api::LangString;
///
/// let json = r#"{"@language": "en", "@value": "Cholera"}"#;
/// let title: LangString = serde_json::from_str(json).unwrap();
/// assert_eq!(title.language.as_deref(), Some("en"));
/// assert_eq!(title.value, "Cholera");
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct LangString {
    /// The BCP 47 language tag, e.g. `"en"`. Optional: absent rather than
    /// failing deserialization if WHO ever omits it.
    #[serde(rename = "@language", default)]
    pub language: Option<String>,
    /// The text value.
    #[serde(rename = "@value", default)]
    pub value: String,
}

/// An ICD-11 Foundation or linearization entity, as returned by
/// [`crate::IcdApiClient::entity`] and [`crate::IcdApiClient::linearization_entity`].
///
/// Field shape per WHO's documented ICD Schema (`specs/who-fic-icd-api.md`):
/// `@id`, `title`, optional `definition`, `parent`/`child` URI lists,
/// optional `browserUrl`, and an optional `code` (present on linearization
/// entities, absent on pure Foundation entities).
///
/// ```
/// use who_fic_icd_api::Entity;
///
/// let json = r#"{
///     "@id": "http://id.who.int/icd/entity/257068234",
///     "title": {"@language": "en", "@value": "Example"},
///     "parent": [],
///     "child": []
/// }"#;
/// let entity: Entity = serde_json::from_str(json).unwrap();
/// assert_eq!(entity.id, "http://id.who.int/icd/entity/257068234");
/// assert_eq!(entity.title.value, "Example");
/// assert!(entity.code.is_none());
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Entity {
    /// The entity's URI, e.g. `"http://id.who.int/icd/entity/257068234"`.
    #[serde(rename = "@id")]
    pub id: String,
    /// The entity's title.
    pub title: LangString,
    /// The entity's definition, if WHO has published one.
    #[serde(default)]
    pub definition: Option<LangString>,
    /// URIs of this entity's parent(s).
    #[serde(default)]
    pub parent: Vec<String>,
    /// URIs of this entity's children.
    #[serde(default)]
    pub child: Vec<String>,
    /// A URL to view this entity in WHO's ICD-11 browser, if present.
    #[serde(rename = "browserUrl", default)]
    pub browser_url: Option<String>,
    /// The classification code, present on linearization entities and
    /// absent on pure Foundation entities.
    #[serde(default)]
    pub code: Option<String>,
}

/// An ICD-10 category, as returned by [`crate::IcdApiClient::icd10_category`].
///
/// A type alias for [`Entity`]: WHO's ICD-10 category responses share the
/// same JSON-LD entity shape (`@id`/`title`/`definition`/`parent`/`child`/
/// `browserUrl`/`code`) documented for ICD-11 entities — `specs/who-fic-icd-api.md`
/// does not document a different shape for it, so this crate does not
/// duplicate the type.
pub type Icd10Entity = Entity;

/// The result of resolving a classification code via
/// [`crate::IcdApiClient::code_info`].
///
/// Beyond the fixed fields (`@id`, `code`, `stemId`, `stemCode`), WHO's
/// postcoordination axis names (`laterality`, `specificAnatomy`, ...) are
/// open-ended — defined per classification, not a fixed set — so they are
/// captured in [`CodeInfo::axes`], a map from axis name to its list of
/// value URIs, rather than as named struct fields.
///
/// Implements [`serde::Deserialize`] via a hand-written impl (see the
/// `impl` block in the crate source) that extracts the known fixed fields
/// first and folds every other top-level array-valued field into `axes`;
/// non-array extra fields are ignored rather than causing a parse failure.
///
/// ```
/// use who_fic_icd_api::CodeInfo;
///
/// let json = r#"{
///     "@id": "http://id.who.int/icd/release/11/2024-01/mms/257068234",
///     "code": "1A00",
///     "laterality": ["http://id.who.int/icd/entity/123"]
/// }"#;
/// let info: CodeInfo = serde_json::from_str(json).unwrap();
/// assert_eq!(info.code, "1A00");
/// assert_eq!(info.axes.get("laterality").unwrap().len(), 1);
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct CodeInfo {
    /// The URI of the resolved entity (the stem entity for a bare code, or
    /// a synthetic cluster URI for a postcoordinated code).
    pub id: String,
    /// The code that was looked up, echoed back by WHO.
    pub code: String,
    /// The URI of the stem entity, if this code is part of a cluster.
    /// Absent for a bare, non-clustered code, where `id` already is the
    /// stem entity.
    pub stem_id: Option<String>,
    /// The stem portion of `code`, if this code is part of a cluster.
    pub stem_code: Option<String>,
    /// Postcoordination axis name (e.g. `"laterality"`, `"specificAnatomy"`)
    /// to its list of value-entity URIs, for every axis WHO included in the
    /// response beyond the fixed fields above.
    pub axes: HashMap<String, Vec<String>>,
}

impl<'de> Deserialize<'de> for CodeInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let serde_json::Value::Object(mut map) = value else {
            return Err(serde::de::Error::custom(
                "expected a JSON object for CodeInfo",
            ));
        };

        let id = map
            .remove("@id")
            .and_then(|v| v.as_str().map(str::to_string))
            .ok_or_else(|| serde::de::Error::missing_field("@id"))?;
        let code = map
            .remove("code")
            .and_then(|v| v.as_str().map(str::to_string))
            .ok_or_else(|| serde::de::Error::missing_field("code"))?;
        let stem_id = map
            .remove("stemId")
            .and_then(|v| v.as_str().map(str::to_string));
        let stem_code = map
            .remove("stemCode")
            .and_then(|v| v.as_str().map(str::to_string));

        let mut axes = HashMap::new();
        for (key, value) in map {
            // Every other WHO-documented top-level field on a CodeInfo
            // response is a postcoordination axis: an array of value URIs.
            // Anything else (e.g. a "@context" string) is not an axis and
            // is permissively ignored rather than causing a parse failure.
            if let serde_json::Value::Array(items) = value {
                let values: Vec<String> = items
                    .into_iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect();
                axes.insert(key, values);
            }
        }

        Ok(CodeInfo {
            id,
            code,
            stem_id,
            stem_code,
            axes,
        })
    }
}

/// One entity in a [`SearchResults::destination_entities`] list.
///
/// WHO's search response shape is not part of the swagger spec's
/// machine-checkable definitions (unlike `Entity`/`CodeInfo`); the fields
/// here are WHO's commonly documented search-result fields. Parsed
/// permissively, matching the rest of this module.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SearchResultEntity {
    /// The matched entity's URI.
    #[serde(alias = "@id")]
    pub id: String,
    /// The matched entity's title, which WHO may include inline HTML
    /// highlighting markup around the matched term(s) in (e.g.
    /// `<em class='found'>Cholera</em>`) — passed through verbatim.
    #[serde(default)]
    pub title: Option<String>,
    /// A relevance score, if WHO included one.
    #[serde(default)]
    pub score: Option<f64>,
    /// The URI of the stem entity, for postcoordinatable results.
    #[serde(rename = "stemId", default)]
    pub stem_id: Option<String>,
    /// The chapter number of the matched entity, if present.
    #[serde(default)]
    pub chapter: Option<String>,
}

/// The result of a search, as returned by
/// [`crate::IcdApiClient::entity_search`] and [`crate::IcdApiClient::search`].
///
/// See [`SearchResultEntity`] for the caveat that this shape is WHO's
/// commonly documented search response, not part of the machine-checkable
/// swagger spec — parsed permissively, like every other type in this
/// module.
///
/// ```
/// use who_fic_icd_api::SearchResults;
///
/// let json = r#"{
///     "destinationEntities": [
///         {"id": "http://id.who.int/icd/entity/257068234", "title": "Example"}
///     ]
/// }"#;
/// let results: SearchResults = serde_json::from_str(json).unwrap();
/// assert_eq!(results.destination_entities.len(), 1);
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SearchResults {
    /// Whether WHO's search endpoint reported an error condition inline
    /// (distinct from an HTTP-level error, which surfaces as
    /// [`crate::IcdApiError::Status`]).
    #[serde(default)]
    pub error: bool,
    /// A human-readable error message, if `error` is `true`.
    #[serde(rename = "errorMessage", default)]
    pub error_message: Option<String>,
    /// The words WHO's search parsed out of the query, if included.
    #[serde(default)]
    pub words: Vec<String>,
    /// The matched entities.
    #[serde(rename = "destinationEntities", default)]
    pub destination_entities: Vec<SearchResultEntity>,
}
