# Spec: `who-fic-icd-api`

An async client for the **WHO ICD-API** (`id.who.int`), the live web
service WHO runs for looking up and searching ICD-10 and ICD-11 entities.
Unlike every other crate in this workspace, this crate makes network calls
— it is the one place credentials are required, and the one place a
non-syntax existence/content question ("what is entity X actually called")
can be answered, because the answer comes from WHO's own servers rather
than data this repository would have to vendor.

Depends on [`who-fic-icd`](who-fic-icd.md) for `Icd10Code`/`Icd11Code` (not
optional — this crate's whole purpose is being their live-data companion).

## Licensing / operational note

The user supplies their own WHO ICD-API credentials (registered at
`icd.who.int/icdapi`, free registration, subject to WHO's terms and rate
limits). This crate is a thin, honest wrapper around WHO's REST API; it
does not cache, redistribute, or bundle any response content beyond a
process-lifetime in-memory OAuth2 token cache.

## Verified against WHO's own OpenAPI spec (2026-08-04)

Fetched directly from `https://id.who.int/swagger/v2/swagger.json` (the
spec backing `https://id.who.int/swagger/index.html`, linked from the
ICD-API v2 docs as "the API-Reference"). Endpoint paths and parameters
below are taken directly from that spec, not guessed.

### Authentication

- Token endpoint: `POST https://icdaccessmanagement.who.int/connect/token`
- Body (`application/x-www-form-urlencoded`): `grant_type=client_credentials`,
  `scope=icdapi_access`, `client_id=...`, `client_secret=...`
- Response JSON: `access_token`, `token_type` (`"Bearer"`), `expires_in`
  (seconds, observed ~3600).
- Every API request thereafter carries `Authorization: Bearer <access_token>`.

### Required headers on every API request

- `Authorization: Bearer <token>`
- `Accept: application/json` (WHO also accepts `application/ld+json`; this
  crate always sends `application/json`)
- `API-Version: v2`
- `Accept-Language: <lang>` (e.g. `en`; configurable, defaults to `en`)

### Endpoints implemented (path : purpose)

| Path | Purpose |
|---|---|
| `GET /icd/entity/{id}` | ICD-11 Foundation entity by its numeric foundation ID |
| `GET /icd/entity/search?q=...` | Foundation-wide search |
| `GET /icd/release/11/{releaseId}/{linearizationname}/{id}` | ICD-11 linearization entity (e.g. `releaseId="2024-01"` or `"latest"`, `linearizationname="mms"`) |
| `GET /icd/release/11/{releaseId}/{linearizationname}/search?q=...` | Search within a linearization |
| `GET /icd/release/11/{releaseId}/{linearizationname}/codeinfo/{code}` | Resolve a classification **code** (not a URI) to its entity, including postcoordination axis breakdown (`stemCode`, plus axis arrays like `laterality`/`specificAnatomy` for cluster codes) — the natural pairing with this workspace's `Icd11Code`/`Cluster` types |
| `GET /icd/release/10/{releaseId}/{code}` | ICD-10 category by code, with its children |

`releaseId`/`linearizationname`/`code`/`id`/query strings are passed
through as plain strings this crate does not further validate — WHO
controls what values are valid, and hardcoding assumptions here (e.g.
which releases exist) would drift. Callers may pass `Icd10Code`/`Icd11Code`
values via `.as_str()`, or raw strings for foundation-only lookups that
don't have a corresponding local type.

Endpoints present in the spec but **not implemented** in v1 of this crate
(document as future extension, don't stub): `/icd/entity/autocode`,
`/icd/release/11/{releaseId}/{linearizationname}/autocode`,
`/icd/release/11/{releaseId}/{linearizationname}/describe`,
`/icd/release/11/{releaseId}/{linearizationname}/lookup`,
`/icd/release/11/{releaseId}/codedit`, `/icd/release/11/{releaseId}/doris`,
the bare `/icd/release/11/{linearizationname}` (no releaseId) variants, and
`POST` search (only `GET` search is implemented; the WHO docs note `POST`
exists only as a workaround for URLs longer than the 2KB `GET` limit).

### Response shapes (WHO's swagger spec does not formally document response
bodies — these are the documented JSON-LD field names from WHO's ICD
Schema and API docs; **parse permissively**: unknown fields ignored,
documented-but-possibly-absent fields optional, never fail deserialization
on an unrecognized field)

`Entity` (foundation or linearization entity response):
- `@id` (URI, string)
- `title` — `{ "@language": str, "@value": str }`
- `definition` — same shape, optional
- `parent` — array of parent URIs (strings)
- `child` — array of child URIs (strings)
- `browserUrl` — string, optional
- `code` — string, present on linearization entities, absent on pure
  foundation entities

`CodeInfo` (from `codeinfo/{code}`):
- `@id` (URI, string)
- `code` — the code that was looked up
- `stemId` — URI of the stem entity, optional (absent for a bare
  non-clustered code where `@id` already is the stem)
- `stemCode` — the stem portion of the code, optional
- other axis fields (`laterality`, `specificAnatomy`, etc.) — WHO's
  postcoordination axis names are open-ended (defined per-classification,
  not a fixed set), so model this as an open map (`HashMap<String,
  Vec<String>>` of axis-name → value-URI-list) rather than named fields,
  after extracting the known fixed fields (`@id`/`code`/`stemId`/`stemCode`)

## Design

### `IcdApiClient`

```rust
pub struct IcdApiClient { /* ... */ }

impl IcdApiClient {
    pub fn builder(client_id: impl Into<String>, client_secret: impl Into<String>)
        -> IcdApiClientBuilder;

    pub async fn entity(&self, foundation_id: &str) -> Result<Entity, IcdApiError>;
    pub async fn linearization_entity(&self, release_id: &str, linearization: &str, id: &str)
        -> Result<Entity, IcdApiError>;
    pub async fn code_info(&self, release_id: &str, linearization: &str, code: &str)
        -> Result<CodeInfo, IcdApiError>;
    pub async fn search(&self, release_id: &str, linearization: &str, query: &str)
        -> Result<SearchResults, IcdApiError>;
    pub async fn icd10_category(&self, release_id: &str, code: &str)
        -> Result<Icd10Entity, IcdApiError>;
}
```

`IcdApiClientBuilder` configures: `client_id`/`client_secret` (required,
taken by `builder()`), `language` (default `"en"`), `token_url` and
`api_base_url` (default the real WHO URLs; overridable — this is the hook
that makes the crate testable without live WHO credentials, by pointing at
a local mock server), and an optional pre-built `reqwest::Client` (so
callers can share a connection pool / configure proxies / TLS).

No blocking/sync API — async only, via `reqwest`, matching modern Rust
HTTP client conventions. The crate does not bundle a Tokio runtime; callers
supply their own (standard practice for async libraries).

### Token management

The client caches its OAuth2 access token in memory (behind a
`tokio::sync::Mutex` or `RwLock`, since multiple concurrent requests must
not each fetch a fresh token) and transparently refetches it once expired
(track `expires_in` against a captured timestamp with a small safety
margin, e.g. refresh 60 seconds before actual expiry). Token fetch
failures surface as `IcdApiError::Auth`.

### `IcdApiError`

`#[non_exhaustive]`, `std::error::Error + Display + Debug`. Variants at
minimum: `Auth` (token fetch/refresh failed), `Http` (transport-level
`reqwest::Error`), `Status { status: u16, body: String }` (a non-2xx
response), `Decode` (response body didn't parse as expected JSON shape).

### Typed-code convenience

Thin wrapper methods/impls so callers with a `who_fic_icd::icd11::Icd11Code`
or `who_fic_icd::icd10::Icd10Code` don't have to call `.as_str()`
themselves — either overloaded-by-type methods or a small extension trait;
implementer's choice, document whichever is chosen.

## Testing

No live WHO credentials are available to this workspace, so the test
suite must not depend on network access to the real WHO API. Use
`wiremock` (a dev-dependency) to run a local mock HTTP server implementing
just enough of the token endpoint and one or two API endpoints to exercise:
token fetch, token caching (second call doesn't re-fetch), token refresh
after simulated expiry, a successful `entity`/`code_info` call parsing a
hand-written fixture response JSON (shaped per the "Response shapes"
section above — do not vendor a real WHO response, WHO's response bodies
are also part of the classification content), a non-2xx response mapping
to `IcdApiError::Status`, and a malformed-JSON response mapping to
`IcdApiError::Decode`.

## Non-goals

- Foundation-wide graph traversal / recursive descendant fetching.
- Any caching beyond the OAuth2 token itself (no response cache — that's
  a reasonable future extension, not in scope now).
- Rate-limit backoff/retry logic (WHO's rate limits are account-tier
  specific and not documented precisely enough to hardcode a policy;
  callers needing this should wrap calls themselves for now).
