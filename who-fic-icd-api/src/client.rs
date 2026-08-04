//! [`IcdApiClient`] and its [`IcdApiClientBuilder`].

use crate::error::IcdApiError;
use crate::types::{CodeInfo, Entity, Icd10Entity, SearchResults};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use who_fic_icd::icd10::Icd10Code;
use who_fic_icd::icd11::Icd11Code;

/// The real WHO OAuth2 token endpoint.
const DEFAULT_TOKEN_URL: &str = "https://icdaccessmanagement.who.int/connect/token";
/// The real WHO ICD-API base URL.
const DEFAULT_API_BASE_URL: &str = "https://id.who.int";
/// The default `Accept-Language` sent on every request.
const DEFAULT_LANGUAGE: &str = "en";
/// The API-Version header value every request carries, per WHO's spec.
const API_VERSION: &str = "v2";
/// How long before a cached token's reported expiry this client refetches
/// it, to avoid racing a request against the token expiring mid-flight.
const TOKEN_SAFETY_MARGIN: Duration = Duration::from_secs(60);

/// An async client for the WHO ICD-API (<https://id.who.int>).
///
/// Handles OAuth2 client-credentials authentication (fetching and caching
/// an access token, transparently refreshing it before it expires) and
/// exposes the subset of WHO's ICD-API endpoints this crate implements —
/// see `specs/who-fic-icd-api.md` for the full list and rationale for what
/// is out of scope.
///
/// Construct one via [`IcdApiClient::builder`]. Cheap to clone-by-reference
/// (methods take `&self`); wrap in an `Arc` to share across tasks if
/// needed.
///
/// # Example
///
/// This example requires real WHO ICD-API credentials, registered for free
/// at <https://icd.who.int/icdapi>, and live network access — so it is
/// marked `no_run` rather than being a runnable doctest.
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use who_fic_icd_api::IcdApiClient;
///
/// let client = IcdApiClient::builder("my-client-id", "my-client-secret").build();
///
/// // Look up a Foundation entity by its numeric ID.
/// let entity = client.entity("257068234").await?;
/// println!("{}", entity.title.value);
///
/// // Resolve an ICD-11 MMS code to its entity and postcoordination axes.
/// let info = client.code_info("2024-01", "mms", "1A00").await?;
/// println!("{}", info.id);
/// # Ok(())
/// # }
/// ```
pub struct IcdApiClient {
    http: reqwest::Client,
    token_url: String,
    api_base_url: String,
    client_id: String,
    client_secret: String,
    language: String,
    token: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

impl IcdApiClient {
    /// Starts building an [`IcdApiClient`] for the given OAuth2
    /// client-credentials (registered at <https://icd.who.int/icdapi>).
    ///
    /// ```
    /// use who_fic_icd_api::IcdApiClient;
    ///
    /// let client = IcdApiClient::builder("id", "secret").build();
    /// ```
    pub fn builder(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> IcdApiClientBuilder {
        IcdApiClientBuilder {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            language: DEFAULT_LANGUAGE.to_string(),
            token_url: DEFAULT_TOKEN_URL.to_string(),
            api_base_url: DEFAULT_API_BASE_URL.to_string(),
            http_client: None,
        }
    }

    /// Fetches an ICD-11 Foundation entity by its numeric foundation ID
    /// (`GET /icd/entity/{id}`).
    ///
    /// `foundation_id` is passed through as a plain string, unvalidated —
    /// see `specs/who-fic-icd-api.md` for why (this crate does not model
    /// Foundation IDs as a typed code).
    pub async fn entity(&self, foundation_id: &str) -> Result<Entity, IcdApiError> {
        let url = format!(
            "{}/icd/entity/{}",
            self.api_base_url,
            percent_encode_path_segment(foundation_id)
        );
        self.get_json(&url).await
    }

    /// Searches the ICD-11 Foundation (`GET /icd/entity/search?q=...`).
    pub async fn entity_search(&self, query: &str) -> Result<SearchResults, IcdApiError> {
        let url = format!(
            "{}/icd/entity/search?q={}",
            self.api_base_url,
            percent_encode_query_value(query)
        );
        self.get_json(&url).await
    }

    /// Fetches an ICD-11 linearization entity (`GET
    /// /icd/release/11/{releaseId}/{linearizationname}/{id}`), e.g.
    /// `release_id = "2024-01"` (or `"latest"`), `linearization = "mms"`.
    pub async fn linearization_entity(
        &self,
        release_id: &str,
        linearization: &str,
        id: &str,
    ) -> Result<Entity, IcdApiError> {
        let url = format!(
            "{}/icd/release/11/{}/{}/{}",
            self.api_base_url,
            percent_encode_path_segment(release_id),
            percent_encode_path_segment(linearization),
            percent_encode_path_segment(id)
        );
        self.get_json(&url).await
    }

    /// Searches within an ICD-11 linearization (`GET
    /// /icd/release/11/{releaseId}/{linearizationname}/search?q=...`).
    pub async fn search(
        &self,
        release_id: &str,
        linearization: &str,
        query: &str,
    ) -> Result<SearchResults, IcdApiError> {
        let url = format!(
            "{}/icd/release/11/{}/{}/search?q={}",
            self.api_base_url,
            percent_encode_path_segment(release_id),
            percent_encode_path_segment(linearization),
            percent_encode_query_value(query)
        );
        self.get_json(&url).await
    }

    /// Resolves a classification code (not a URI) to its entity and
    /// postcoordination axis breakdown (`GET
    /// /icd/release/11/{releaseId}/{linearizationname}/codeinfo/{code}`).
    ///
    /// For callers holding a typed [`who_fic_icd::icd11::Icd11Code`], see
    /// [`IcdApiClient::code_info_typed`].
    pub async fn code_info(
        &self,
        release_id: &str,
        linearization: &str,
        code: &str,
    ) -> Result<CodeInfo, IcdApiError> {
        let url = format!(
            "{}/icd/release/11/{}/{}/codeinfo/{}",
            self.api_base_url,
            percent_encode_path_segment(release_id),
            percent_encode_path_segment(linearization),
            percent_encode_path_segment(code)
        );
        self.get_json(&url).await
    }

    /// Convenience wrapper over [`IcdApiClient::code_info`] for callers
    /// holding a typed [`Icd11Code`], instead of calling `.as_str()`
    /// themselves.
    pub async fn code_info_typed(
        &self,
        release_id: &str,
        linearization: &str,
        code: &Icd11Code,
    ) -> Result<CodeInfo, IcdApiError> {
        self.code_info(release_id, linearization, code.as_str())
            .await
    }

    /// Fetches an ICD-10 category by code, with its children (`GET
    /// /icd/release/10/{releaseId}/{code}`).
    ///
    /// For callers holding a typed [`who_fic_icd::icd10::Icd10Code`], see
    /// [`IcdApiClient::icd10_category_typed`].
    pub async fn icd10_category(
        &self,
        release_id: &str,
        code: &str,
    ) -> Result<Icd10Entity, IcdApiError> {
        let url = format!(
            "{}/icd/release/10/{}/{}",
            self.api_base_url,
            percent_encode_path_segment(release_id),
            percent_encode_path_segment(code)
        );
        self.get_json(&url).await
    }

    /// Convenience wrapper over [`IcdApiClient::icd10_category`] for
    /// callers holding a typed [`Icd10Code`], instead of calling
    /// `.as_str()` themselves.
    pub async fn icd10_category_typed(
        &self,
        release_id: &str,
        code: &Icd10Code,
    ) -> Result<Icd10Entity, IcdApiError> {
        self.icd10_category(release_id, code.as_str()).await
    }

    /// Performs an authenticated `GET` against `url`, decoding a
    /// successful response as `T`.
    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, IcdApiError> {
        let token = self.access_token().await?;
        let response = self
            .http
            .get(url)
            .bearer_auth(token)
            .header(reqwest::header::ACCEPT, "application/json")
            .header("API-Version", API_VERSION)
            .header("Accept-Language", &self.language)
            .send()
            .await
            .map_err(IcdApiError::Http)?;

        let status = response.status();
        let body = response.text().await.map_err(IcdApiError::Http)?;
        if !status.is_success() {
            return Err(IcdApiError::Status {
                status: status.as_u16(),
                body,
            });
        }
        serde_json::from_str(&body).map_err(IcdApiError::Decode)
    }

    /// Returns a valid access token, using the cached one if it has not yet
    /// reached its safety-margin expiry, otherwise fetching (and caching) a
    /// fresh one.
    ///
    /// Holds the token mutex across the fetch so concurrent callers
    /// serialize on it rather than each triggering their own token
    /// request: the first caller through the lock (per expiry) fetches and
    /// caches; everyone else either finds a still-valid cached token or
    /// waits their turn.
    async fn access_token(&self) -> Result<String, IcdApiError> {
        let mut guard = self.token.lock().await;
        if let Some(cached) = guard.as_ref() {
            if Instant::now() < cached.expires_at {
                return Ok(cached.access_token.clone());
            }
        }

        let response = self.fetch_token().await?;
        let expires_in = Duration::from_secs(response.expires_in);
        let expires_at = Instant::now() + expires_in.saturating_sub(TOKEN_SAFETY_MARGIN);
        let access_token = response.access_token;
        *guard = Some(CachedToken {
            access_token: access_token.clone(),
            expires_at,
        });
        Ok(access_token)
    }

    /// Fetches a fresh access token from the token endpoint.
    async fn fetch_token(&self) -> Result<TokenResponse, IcdApiError> {
        let body = form_urlencode(&[
            ("grant_type", "client_credentials"),
            ("scope", "icdapi_access"),
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
        ]);

        let response = self
            .http
            .post(&self.token_url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await
            .map_err(|source| IcdApiError::Auth(source.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|source| IcdApiError::Auth(source.to_string()))?;
        if !status.is_success() {
            return Err(IcdApiError::Auth(format!(
                "token endpoint returned HTTP {status}: {body}"
            )));
        }
        serde_json::from_str(&body)
            .map_err(|source| IcdApiError::Auth(format!("malformed token response: {source}")))
    }
}

/// Builds an [`IcdApiClient`].
///
/// Created via [`IcdApiClient::builder`]. `client_id`/`client_secret` are
/// required and taken by `builder()` itself; everything else has a default
/// suitable for talking to the real WHO ICD-API and is overridden only when
/// needed — in particular, [`IcdApiClientBuilder::token_url`] and
/// [`IcdApiClientBuilder::api_base_url`] are how tests point this client at
/// a local mock server instead of WHO's live service.
pub struct IcdApiClientBuilder {
    client_id: String,
    client_secret: String,
    language: String,
    token_url: String,
    api_base_url: String,
    http_client: Option<reqwest::Client>,
}

impl IcdApiClientBuilder {
    /// Sets the `Accept-Language` sent on every API request. Defaults to
    /// `"en"`.
    ///
    /// ```
    /// use who_fic_icd_api::IcdApiClient;
    ///
    /// let client = IcdApiClient::builder("id", "secret").language("fr").build();
    /// ```
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Overrides the OAuth2 token endpoint URL. Defaults to WHO's real
    /// token endpoint; overriding this is how tests point the client at a
    /// local mock server.
    ///
    /// ```
    /// use who_fic_icd_api::IcdApiClient;
    ///
    /// let client = IcdApiClient::builder("id", "secret")
    ///     .token_url("http://127.0.0.1:0/connect/token")
    ///     .build();
    /// ```
    pub fn token_url(mut self, token_url: impl Into<String>) -> Self {
        self.token_url = token_url.into();
        self
    }

    /// Overrides the ICD-API base URL (default `https://id.who.int`).
    /// Overriding this is how tests point the client at a local mock
    /// server.
    ///
    /// ```
    /// use who_fic_icd_api::IcdApiClient;
    ///
    /// let client = IcdApiClient::builder("id", "secret")
    ///     .api_base_url("http://127.0.0.1:0")
    ///     .build();
    /// ```
    pub fn api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = api_base_url.into();
        self
    }

    /// Supplies a pre-built [`reqwest::Client`] (e.g. to share a connection
    /// pool, configure a proxy, or set custom TLS options) instead of
    /// letting [`IcdApiClientBuilder::build`] construct a default one.
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Builds the [`IcdApiClient`].
    ///
    /// ```
    /// use who_fic_icd_api::IcdApiClient;
    ///
    /// let client = IcdApiClient::builder("id", "secret").build();
    /// ```
    pub fn build(self) -> IcdApiClient {
        IcdApiClient {
            http: self.http_client.unwrap_or_default(),
            token_url: trim_trailing_slash(self.token_url),
            api_base_url: trim_trailing_slash(self.api_base_url),
            client_id: self.client_id,
            client_secret: self.client_secret,
            language: self.language,
            token: Mutex::new(None),
        }
    }
}

fn trim_trailing_slash(mut s: String) -> String {
    while s.ends_with('/') {
        s.pop();
    }
    s
}

/// Percent-encodes a single URL path segment (unreserved characters per
/// RFC 3986 pass through; everything else becomes `%XX`).
///
/// This crate has no dependency capable of doing this for it (no `url` or
/// `percent-encoding` crate, per `specs/architecture.md`'s dependency
/// discipline, and `reqwest`'s own helpers for this are gated behind the
/// `query`/`form` features this crate does not enable) — the encoding rules
/// are small and fixed, so a hand-written encoder is the same tradeoff this
/// workspace already makes for code-parsing grammars.
fn percent_encode_path_segment(input: &str) -> String {
    percent_encode(input)
}

/// Percent-encodes a URL query *value* (same rule as a path segment for
/// this crate's purposes: WHO's `q` parameter is passed as a normal
/// percent-encoded query value, not `application/x-www-form-urlencoded`
/// space-as-`+` encoding, which only applies to request bodies).
fn percent_encode_query_value(input: &str) -> String {
    percent_encode(input)
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{byte:02X}"));
            }
        }
    }
    out
}

/// Encodes `pairs` as an `application/x-www-form-urlencoded` body: percent
/// encoding as above, but with a literal space encoded as `+` (per the
/// `application/x-www-form-urlencoded` spec, rather than a URL query's
/// `%20`), joined with `&`.
fn form_urlencode(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                percent_encode(key).replace("%20", "+"),
                percent_encode(value).replace("%20", "+")
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_leaves_unreserved_characters_alone() {
        assert_eq!(percent_encode("abc-XYZ_123.~"), "abc-XYZ_123.~");
    }

    #[test]
    fn percent_encode_escapes_reserved_characters() {
        assert_eq!(percent_encode("a b&c"), "a%20b%26c");
    }

    #[test]
    fn form_urlencode_uses_plus_for_space() {
        let body = form_urlencode(&[("grant_type", "client credentials")]);
        assert_eq!(body, "grant_type=client+credentials");
    }

    #[test]
    fn form_urlencode_joins_multiple_pairs_with_ampersand() {
        let body = form_urlencode(&[("a", "1"), ("b", "2")]);
        assert_eq!(body, "a=1&b=2");
    }

    #[test]
    fn trim_trailing_slash_removes_all_trailing_slashes() {
        assert_eq!(
            trim_trailing_slash("https://id.who.int/".to_string()),
            "https://id.who.int"
        );
        assert_eq!(
            trim_trailing_slash("https://id.who.int".to_string()),
            "https://id.who.int"
        );
    }
}
