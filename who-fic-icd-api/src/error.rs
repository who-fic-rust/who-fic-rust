//! [`IcdApiError`], the single error type returned by every fallible
//! [`crate::IcdApiClient`] method.

use std::fmt;

/// An error returned by an [`crate::IcdApiClient`] operation.
///
/// `#[non_exhaustive]` so new variants can be added without a breaking
/// change. Implements [`std::error::Error`], so it composes with `?` and
/// error-reporting crates the normal way.
///
/// ```
/// use who_fic_icd_api::IcdApiError;
///
/// let err = IcdApiError::Status { status: 404, body: "not found".to_string() };
/// match err {
///     IcdApiError::Status { status, .. } => assert_eq!(status, 404),
///     _ => unreachable!(),
/// }
/// ```
#[non_exhaustive]
#[derive(Debug)]
pub enum IcdApiError {
    /// Fetching or refreshing the OAuth2 access token failed — a transport
    /// error talking to the token endpoint, a non-2xx response from it, or
    /// a token response body that didn't parse as expected. The message
    /// describes what went wrong.
    Auth(String),
    /// A transport-level error (connection, TLS, timeout, ...) from
    /// `reqwest` while making an API request.
    Http(reqwest::Error),
    /// The API responded with a non-2xx HTTP status. `body` is the raw
    /// response body (which may be empty, plain text, or a WHO error JSON
    /// payload — this crate does not attempt to parse it further).
    Status {
        /// The HTTP status code, e.g. `404`.
        status: u16,
        /// The raw response body.
        body: String,
    },
    /// The response body did not deserialize into the expected shape.
    Decode(serde_json::Error),
}

impl fmt::Display for IcdApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth(message) => write!(f, "WHO ICD-API authentication failed: {message}"),
            Self::Http(source) => write!(f, "WHO ICD-API request failed: {source}"),
            Self::Status { status, body } => {
                write!(f, "WHO ICD-API returned HTTP {status}: {body}")
            }
            Self::Decode(source) => {
                write!(f, "failed to decode WHO ICD-API response: {source}")
            }
        }
    }
}

impl std::error::Error for IcdApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(source) => Some(source),
            Self::Decode(source) => Some(source),
            Self::Auth(_) | Self::Status { .. } => None,
        }
    }
}
