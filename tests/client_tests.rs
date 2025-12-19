//! Integration tests for the UFM client.

use serde_json::json;
use ufm_client::{Auth, Error, UfmClient};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a test client connected to the mock server.
async fn create_test_client(mock_server: &MockServer) -> UfmClient {
    UfmClient::builder()
        .base_url(mock_server.uri())
        .auth(Auth::basic("admin", "password"))
        .build()
        .expect("Failed to create test client")
}

#[tokio::test]
async fn test_get_version() {
    let mock_server = MockServer::start().await;

    let response = json!({
        "opensm": "5.20.0_ef1f438",
        "sharp": "3.8.0-rc5",
        "ibdiagnet": "2.18.0",
        "telemetry": "1.0.0",
        "mft": "4.28.0-83",
        "ufm_webui": "6.18.0-5",
        "ufm_release_version": "6.18.0-5",
        "plugins": {
            "ufm_syslog_streaming_plugin": "1.0.0"
        }
    });

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/app/ufm_version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let version = client.version().await.expect("Failed to get version");

    assert_eq!(version.ufm_release_version, "6.18.0-5");
    assert_eq!(version.opensm, Some("5.20.0_ef1f438".to_string()));
    assert_eq!(version.major_version(), Some(6));
    assert_eq!(version.minor_version(), Some(18));
    assert!(version.is_at_least(6, 17));
    assert!(!version.is_at_least(6, 19));
}

#[tokio::test]
async fn test_authentication_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/pkeys"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid credentials"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let result = client.pkeys().list().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_auth_error());
}

#[tokio::test]
async fn test_not_found_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/pkeys/0x99"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "PKey 0x99 does not exist"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkey = "0x99".parse().unwrap();
    let result = client.pkeys().get(&pkey).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { resource } => {
            assert!(resource.contains("Not Found") || resource.contains("PKey"));
        }
        other => panic!("Expected NotFound error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_client_builder_validation() {
    // Missing base_url should fail
    let result = UfmClient::builder()
        .auth(Auth::basic("admin", "pass"))
        .build();
    assert!(result.is_err());

    // Invalid URL should fail
    let result = UfmClient::builder()
        .base_url("not-a-valid-url")
        .auth(Auth::basic("admin", "pass"))
        .build();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_auth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/pkeys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(["0x12"])))
        .mount(&mock_server)
        .await;

    let client = UfmClient::builder()
        .base_url(mock_server.uri())
        .auth(Auth::token("my-access-token"))
        .build()
        .expect("Failed to create client");

    let pkeys = client.pkeys().list().await.expect("Failed to list pkeys");
    assert_eq!(pkeys.len(), 1);
}
