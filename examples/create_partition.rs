//! Example: Create a new partition key.
//!
//! This example demonstrates how to create a new partition key (PKey) in UFM
//! with various configuration options.
//!
//! Run with: cargo run --example create_partition
//!
//! Environment variables:
//! - UFM_HOST: The UFM server hostname (default: localhost)
//! - UFM_USER: The UFM username (default: admin)
//! - UFM_PASSWORD: The UFM password (default: admin)
//! - PKEY: The partition key to create (default: 0x42)

use std::env;

use ufm_client::types::Pkey;
use ufm_client::{Auth, UfmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment
    let host = env::var("UFM_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("UFM_USER").unwrap_or_else(|_| "admin".to_string());
    let pass = env::var("UFM_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    let pkey_str = env::var("PKEY").unwrap_or_else(|_| "0x42".to_string());

    let base_url = format!("https://{host}");
    println!("Connecting to UFM at {base_url}...");

    // Create the client
    let client = UfmClient::builder()
        .base_url(&base_url)
        .auth(Auth::basic(&user, &pass))
        .verify_ssl(false) // For self-signed certificates
        .build()?;

    // Parse the PKey from the environment or default
    let pkey: Pkey = pkey_str.parse()?;
    println!("\n=== Creating Partition Key {pkey} ===\n");

    // Check if the PKey already exists
    let existing_pkeys = client.pkeys().list().await?;
    if existing_pkeys.contains(&pkey) {
        println!("Partition key {pkey} already exists!");
        println!("To delete it first, run: cargo run --example delete_partition");
        return Ok(());
    }

    // Create the partition with configuration options
    client
        .pkeys()
        .create(&pkey)
        .ip_over_ib(true) // Enable IP over InfiniBand
        .mtu_limit(4) // Set MTU limit (1-5, where 4 = 2048 bytes)
        .service_level(0) // Set service level (0-15)
        .rate_limit(100.0) // Set rate limit in Gbps
        .send()
        .await?;

    println!("Successfully created partition key: {pkey}");

    // Verify the partition was created
    let details = client.pkeys().get(&pkey).await?;
    println!("\nPartition details:");
    println!("  Name: {}", details.partition);
    println!("  IP over IB: {}", details.ip_over_ib);
    println!("  Members: {} GUIDs", details.guids.len());

    if let Some(qos) = &details.qos_conf {
        if let Some(mtu) = qos.mtu_limit {
            println!("  MTU limit: {mtu}");
        }
        if let Some(sl) = qos.service_level {
            println!("  Service level: {sl}");
        }
        if let Some(rate) = qos.rate_limit {
            println!("  Rate limit: {rate} Gbps");
        }
    }

    println!("\nDone! To add GUIDs, run: cargo run --example add_guids_to_partition");

    Ok(())
}
