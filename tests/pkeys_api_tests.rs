//! Integration tests for the PKeys API using wiremock.

use serde_json::json;
use ufm_client::types::{Guid, Membership, Pkey};
use ufm_client::{Auth, UfmClient};
use wiremock::matchers::{body_json, method, path};
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
async fn test_list_pkeys() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/pkeys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(["0x12", "0x13", "0x7fff"])))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkeys = client.pkeys().list().await.expect("Failed to list pkeys");

    assert_eq!(pkeys.len(), 3);
    assert_eq!(pkeys[0].value(), 0x12);
    assert_eq!(pkeys[1].value(), 0x13);
    assert_eq!(pkeys[2].value(), 0x7fff);
}

#[tokio::test]
async fn test_get_pkey_details() {
    let mock_server = MockServer::start().await;

    let response = json!({
        "partition": "test_partition",
        "ip_over_ib": true,
        "guids": [
            {"guid": "0002c903000e0b72", "membership": "full", "index0": false},
            {"guid": "0002c903000e0b73", "membership": "limited", "index0": false}
        ]
    });

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/pkeys/0x12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkey: Pkey = "0x12".parse().unwrap();
    let details = client.pkeys().get(&pkey).await.expect("Failed to get pkey");

    assert_eq!(details.partition, "test_partition");
    assert!(details.ip_over_ib);
    assert_eq!(details.guids.len(), 2);
    assert!(details.guids[0].membership.is_full());
    assert!(details.guids[1].membership.is_limited());
}

#[tokio::test]
async fn test_get_pkey_missing_returns_not_found() {
    let mock_server = MockServer::start().await;

    // UFM answers a nonexistent pkey with `200 {}` rather than 404.
    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/pkeys/0x99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkey: Pkey = "0x99".parse().unwrap();
    let err = client.pkeys().get(&pkey).await.unwrap_err();
    assert!(err.is_not_found(), "expected NotFound, got {err:?}");
}

#[tokio::test]
async fn test_create_pkey() {
    let mock_server = MockServer::start().await;

    // Creating an empty pkey uses the /resources/pkeys/add endpoint
    Mock::given(method("POST"))
        .and(path("/ufmRestV2/resources/pkeys/add"))
        .and(body_json(json!({
            "pkey": "0x12",
            "ip_over_ib": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkey: Pkey = "0x12".parse().unwrap();

    client
        .pkeys()
        .create(&pkey)
        .ip_over_ib(true)
        .send()
        .await
        .expect("Failed to create pkey");
}

#[tokio::test]
async fn test_add_guids_to_pkey() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ufmRestV2/resources/pkeys"))
        .and(body_json(json!({
            "pkey": "0x12",
            "guids": ["0002c903000e0b72", "0002c903000e0b73"],
            "membership": "full"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkey: Pkey = "0x12".parse().unwrap();
    let guid1: Guid = "0002c903000e0b72".parse().unwrap();
    let guid2: Guid = "0002c903000e0b73".parse().unwrap();

    client
        .pkeys()
        .add_guids(&pkey, &[guid1, guid2])
        .membership(Membership::Full)
        .send()
        .await
        .expect("Failed to add guids");
}

#[tokio::test]
async fn test_remove_guids_from_pkey() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/ufmRestV2/resources/pkeys"))
        .and(body_json(json!({
            "pkey": "0x12",
            "guids": ["0002c903000e0b72"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkey: Pkey = "0x12".parse().unwrap();
    let guid: Guid = "0002c903000e0b72".parse().unwrap();

    client
        .pkeys()
        .remove_guids(&pkey, &[guid])
        .await
        .expect("Failed to remove guids");
}

#[tokio::test]
async fn test_delete_pkey() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/ufmRestV2/resources/pkeys/0x12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let pkey: Pkey = "0x12".parse().unwrap();

    client
        .pkeys()
        .delete(&pkey)
        .await
        .expect("Failed to delete pkey");
}
