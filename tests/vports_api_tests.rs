//! Integration tests for the VPorts API using wiremock.

use serde_json::json;
use ufm_client::types::VportState;
use ufm_client::{Auth, UfmClient};
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
async fn test_list_vports() {
    let mock_server = MockServer::start().await;

    let response = json!([
        {
            "virtual_port_state": "Active",
            "virtual_port_lid": 3434,
            "system_ip": "11.4.3.175",
            "system_name": "r-ufm77",
            "node_description": "r-ufm77 mlx5_0",
            "system_guid": "248a0703008a850a",
            "port_guid": "248a0703008a850a",
            "port_name": "248a0703008a850a_1",
            "physical_port_number": 1,
            "virtual_port_guid": "0002c90000000017"
        },
        {
            "virtual_port_state": "Down",
            "virtual_port_lid": 3435,
            "system_ip": "11.4.3.176",
            "system_name": "r-ufm78",
            "node_description": "r-ufm78 mlx5_0",
            "system_guid": "248a0703008a850b",
            "port_guid": "248a0703008a850b",
            "port_name": "248a0703008a850b_1",
            "physical_port_number": 1,
            "virtual_port_guid": "0002c90000000018"
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/vports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let vports = client.vports().list().await.expect("Failed to list vports");

    assert_eq!(vports.len(), 2);
    assert_eq!(vports[0].virtual_port_guid, "0002c90000000017");
    assert_eq!(vports[0].state, VportState::Active);
    assert_eq!(vports[0].system_name, "r-ufm77");
    assert!(vports[0].is_active());

    assert_eq!(vports[1].virtual_port_guid, "0002c90000000018");
    assert_eq!(vports[1].state, VportState::Down);
    assert!(!vports[1].is_active());
}

#[tokio::test]
async fn test_virtualization_enabled() {
    let mock_server = MockServer::start().await;

    // Return an empty list - virtualization is enabled but no vports
    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/vports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let enabled = client
        .vports()
        .is_virtualization_enabled()
        .await
        .expect("Failed to check virtualization");

    assert!(enabled);
}

#[tokio::test]
async fn test_virtualization_not_enabled_404() {
    let mock_server = MockServer::start().await;

    // Return 404 - virtualization not enabled
    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/vports"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Resource not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let enabled = client
        .vports()
        .is_virtualization_enabled()
        .await
        .expect("Failed to check virtualization");

    assert!(!enabled);
}

#[tokio::test]
async fn test_by_system_guid() {
    let mock_server = MockServer::start().await;

    let response = json!([
        {
            "virtual_port_state": "Active",
            "virtual_port_lid": 3434,
            "system_ip": "11.4.3.175",
            "system_name": "r-ufm77",
            "node_description": "r-ufm77 mlx5_0",
            "system_guid": "248a0703008a850a",
            "port_guid": "248a0703008a850a",
            "port_name": "248a0703008a850a_1",
            "physical_port_number": 1,
            "virtual_port_guid": "0002c90000000017"
        },
        {
            "virtual_port_state": "Active",
            "virtual_port_lid": 3435,
            "system_ip": "11.4.3.176",
            "system_name": "r-ufm78",
            "node_description": "r-ufm78 mlx5_0",
            "system_guid": "248a0703008a850b",
            "port_guid": "248a0703008a850b",
            "port_name": "248a0703008a850b_1",
            "physical_port_number": 1,
            "virtual_port_guid": "0002c90000000018"
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/ufmRestV2/resources/vports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let vports = client
        .vports()
        .by_system_guid("248a0703008a850a")
        .await
        .expect("Failed to filter vports");

    assert_eq!(vports.len(), 1);
    assert_eq!(vports[0].system_guid, "248a0703008a850a");
}
