//! Integration tests against a local `wiremock` server standing in for
//! WHO's real token and ICD-API endpoints — no live WHO credentials or
//! network access are available to this workspace, so every test here is
//! self-contained. See `specs/who-fic-icd-api.md`'s "Testing" section for
//! what this suite is required to cover.

use who_fic_icd_api::{IcdApiClient, IcdApiError};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mounts a token endpoint that always succeeds, returning `expires_in`
/// seconds of validity, and returns the mock's own request-matching guard
/// so the caller can assert on how many times it was hit.
async fn mount_token_endpoint(server: &MockServer, expires_in: u64) {
    Mock::given(method("POST"))
        .and(path("/connect/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "test-access-token",
            "token_type": "Bearer",
            "expires_in": expires_in,
        })))
        .mount(server)
        .await;
}

async fn token_hits(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .expect("wiremock request recording enabled")
        .iter()
        .filter(|req| req.url.path() == "/connect/token")
        .count()
}

fn test_client(server: &MockServer) -> IcdApiClient {
    IcdApiClient::builder("test-client-id", "test-client-secret")
        .token_url(format!("{}/connect/token", server.uri()))
        .api_base_url(server.uri())
        .build()
}

const FIXTURE_ENTITY: &str = r#"{
    "@id": "http://id.who.int/icd/entity/257068234",
    "title": {"@language": "en", "@value": "Example"},
    "parent": [],
    "child": []
}"#;

#[tokio::test]
async fn fetches_token_on_first_request() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/257068234"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE_ENTITY))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let entity = client.entity("257068234").await.unwrap();

    assert_eq!(entity.id, "http://id.who.int/icd/entity/257068234");
    assert_eq!(token_hits(&server).await, 1);
}

#[tokio::test]
async fn reuses_cached_token_on_second_request() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/257068234"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE_ENTITY))
        .mount(&server)
        .await;

    let client = test_client(&server);
    client.entity("257068234").await.unwrap();
    client.entity("257068234").await.unwrap();

    // Two API calls, but the token endpoint should only have been hit once
    // — the second call must reuse the cached token.
    assert_eq!(token_hits(&server).await, 1);
}

#[tokio::test]
async fn refreshes_token_after_simulated_expiry() {
    let server = MockServer::start().await;
    // A 1-second token lifetime; the client's safety margin (60s) means it
    // is treated as immediately due for refresh on next use, but we still
    // sleep past the literal expiry to exercise real elapsed-time behavior.
    mount_token_endpoint(&server, 1).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/257068234"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE_ENTITY))
        .mount(&server)
        .await;

    let client = test_client(&server);
    client.entity("257068234").await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    client.entity("257068234").await.unwrap();

    assert_eq!(token_hits(&server).await, 2);
}

#[tokio::test]
async fn required_headers_are_sent_on_api_requests() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/257068234"))
        .and(header("Authorization", "Bearer test-access-token"))
        .and(header("Accept", "application/json"))
        .and(header("API-Version", "v2"))
        .and(header("Accept-Language", "en"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE_ENTITY))
        .mount(&server)
        .await;

    let client = test_client(&server);
    // If any required header were missing, the request would not match the
    // mock above and wiremock would respond 404, which entity() would
    // surface as IcdApiError::Status — so a successful result here proves
    // every header matcher was satisfied.
    client.entity("257068234").await.unwrap();
}

#[tokio::test]
async fn custom_language_is_sent() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/257068234"))
        .and(header("Accept-Language", "fr"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE_ENTITY))
        .mount(&server)
        .await;

    let client = IcdApiClient::builder("test-client-id", "test-client-secret")
        .token_url(format!("{}/connect/token", server.uri()))
        .api_base_url(server.uri())
        .language("fr")
        .build();
    client.entity("257068234").await.unwrap();
}

#[tokio::test]
async fn code_info_parses_fixture_with_postcoordination_axes() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/release/11/2024-01/mms/codeinfo/1A00.0"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{
                "@id": "http://id.who.int/icd/release/11/2024-01/mms/257068234/other",
                "code": "1A00.0",
                "stemId": "http://id.who.int/icd/release/11/2024-01/mms/257068234",
                "stemCode": "1A00",
                "laterality": ["http://id.who.int/icd/entity/111"]
            }"#,
        ))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let info = client.code_info("2024-01", "mms", "1A00.0").await.unwrap();

    assert_eq!(info.code, "1A00.0");
    assert_eq!(info.stem_code.as_deref(), Some("1A00"));
    assert_eq!(
        info.axes.get("laterality").unwrap(),
        &vec!["http://id.who.int/icd/entity/111".to_string()]
    );
}

#[tokio::test]
async fn non_2xx_response_maps_to_status_error() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/999999999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let err = client.entity("999999999").await.unwrap_err();

    match err {
        IcdApiError::Status { status, body } => {
            assert_eq!(status, 404);
            assert_eq!(body, "not found");
        }
        other => panic!("expected IcdApiError::Status, got {other:?}"),
    }
}

#[tokio::test]
async fn malformed_json_response_maps_to_decode_error() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/257068234"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ not valid json"))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let err = client.entity("257068234").await.unwrap_err();

    assert!(matches!(err, IcdApiError::Decode(_)), "got {err:?}");
}

#[tokio::test]
async fn token_endpoint_failure_maps_to_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/connect/token"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid_client"))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let err = client.entity("257068234").await.unwrap_err();

    assert!(matches!(err, IcdApiError::Auth(_)), "got {err:?}");
}

#[tokio::test]
async fn icd10_category_uses_icd10_entity_alias() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/release/10/2016/A00"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{
                "@id": "http://id.who.int/icd/release/10/2016/A00",
                "title": {"@language": "en", "@value": "Example category"},
                "parent": [],
                "child": [],
                "code": "A00"
            }"#,
        ))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let category = client.icd10_category("2016", "A00").await.unwrap();

    assert_eq!(category.code.as_deref(), Some("A00"));
}

#[tokio::test]
async fn typed_convenience_wrappers_accept_typed_codes() {
    use std::str::FromStr;
    use who_fic_icd::icd10::Icd10Code;
    use who_fic_icd::icd11::Icd11Code;

    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/release/11/2024-01/mms/codeinfo/1A00"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"@id": "http://id.who.int/icd/entity/1", "code": "1A00"}"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/icd/release/10/2016/A00"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE_ENTITY))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let icd11_code = Icd11Code::from_str("1A00").unwrap();
    let icd10_code = Icd10Code::from_str("A00").unwrap();

    let info = client
        .code_info_typed("2024-01", "mms", &icd11_code)
        .await
        .unwrap();
    assert_eq!(info.code, "1A00");

    client
        .icd10_category_typed("2016", &icd10_code)
        .await
        .unwrap();
}

#[tokio::test]
async fn entity_search_hits_foundation_search_endpoint() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/entity/search"))
        .and(query_param("q", "cholera"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"destinationEntities": [{"id": "http://id.who.int/icd/entity/257068234", "title": "Example"}]}"#,
        ))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let results = client.entity_search("cholera").await.unwrap();

    assert_eq!(results.destination_entities.len(), 1);
}

#[tokio::test]
async fn linearization_search_hits_linearization_search_endpoint() {
    let server = MockServer::start().await;
    mount_token_endpoint(&server, 3600).await;
    Mock::given(method("GET"))
        .and(path("/icd/release/11/2024-01/mms/search"))
        .and(query_param("q", "cholera"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"destinationEntities": []}"#))
        .mount(&server)
        .await;

    let client = test_client(&server);
    let results = client.search("2024-01", "mms", "cholera").await.unwrap();

    assert!(results.destination_entities.is_empty());
}
