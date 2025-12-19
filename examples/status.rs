//! Basic usage example for the UFM client library.
//!
//! This example demonstrates how to:
//! - Create a client with authentication
//! - Get UFM version information
//! - Check if virtualization is enabled and list vports
//! - List and manage partition keys (PKeys)
//! - Add and remove GUIDs from partitions
//!
//! Run with: cargo run --example basic_usage
//!
//! For debug logging, set RUST_LOG=debug:
//!   RUST_LOG=debug cargo run --example basic_usage
//!
//! Note: This example requires a running UFM instance.
//! Set the following environment variables:
//! - UFM_HOST: The UFM server hostname (default: localhost)
//! - UFM_USER: The UFM username (default: admin)
//! - UFM_PASSWORD: The UFM password (default: admin)

use std::env;

use tracing_subscriber::EnvFilter;
use ufm_client::{Auth, UfmClient};

// These are used in the commented-out examples below
#[allow(unused_imports)]
use ufm_client::types::{Guid, Membership, Pkey, QosConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with env filter
    // Set RUST_LOG=debug to see detailed request/response info
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Get configuration from environment
    let host = env::var("UFM_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("UFM_USER").unwrap_or_else(|_| "admin".to_string());
    let pass = env::var("UFM_PASSWORD").unwrap_or_else(|_| "admin".to_string());

    let base_url = format!("https://{host}");

    println!("Connecting to UFM at {base_url}...");

    // Create the client
    let client = UfmClient::builder()
        .base_url(&base_url)
        .auth(Auth::basic(&user, &pass))
        .verify_ssl(false) // For self-signed certificates
        .build()?;

    // Get UFM version
    println!("\n=== UFM Version ===");
    let version = client.version().await?;
    println!("UFM Release: {}", version.ufm_release_version);
    if let Some(opensm) = &version.opensm {
        println!("OpenSM: {opensm}");
    }

    // Check if virtualization is enabled
    println!("\n=== Virtualization Status ===");
    match client.vports().is_virtualization_enabled().await {
        Ok(true) => {
            println!("Virtualization is ENABLED");
            // List vports
            let vports = client.vports().list().await?;
            println!("Found {} virtual ports:", vports.len());
            for vport in vports.iter().take(5) {
                println!(
                    "  - {} on {} ({})",
                    vport.virtual_port_guid, vport.system_name, vport.state
                );
            }
            if vports.len() > 5 {
                println!("  ... and {} more", vports.len() - 5);
            }
        }
        Ok(false) => {
            println!("Virtualization is NOT enabled on this subnet manager");
        }
        Err(e) => {
            println!("Could not check virtualization status: {e}");
        }
    }

    // List all partition keys
    println!("\n=== Partition Keys ===");
    let pkeys = client.pkeys().list().await?;
    println!("Found {} partition keys:", pkeys.len());
    for pkey in &pkeys {
        println!("  - {pkey}");
    }

    // Example: Create a new partition key (commented out to avoid side effects)
    /*
    println!("\n=== Creating PKey 0x42 ===");
    let new_pkey: Pkey = "0x42".parse()?;
    client.pkeys()
        .create(&new_pkey)
        .ip_over_ib(true)
        .send()
        .await?;
    println!("Created partition key: {}", new_pkey);
    */

    // Example: Add GUIDs to a partition (commented out to avoid side effects)
    /*
    println!("\n=== Adding GUIDs to PKey ===");
    let guid1: Guid = "0002c903000e0b72".parse()?;
    let guid2: Guid = "0002c903000e0b73".parse()?;
    client.pkeys()
        .add_guids(&new_pkey, &[guid1, guid2])
        .membership(Membership::Full)
        .ip_over_ib(true)
        .send()
        .await?;
    println!("Added 2 GUIDs with full membership");
    */

    // Get details of the first partition key (if any)
    if let Some(first_pkey) = pkeys.first() {
        println!("\n=== Details for {first_pkey} ===");
        let details = client.pkeys().get(first_pkey).await?;
        println!("Partition name: {}", details.partition);
        println!("IP over IB: {}", details.ip_over_ib);
        println!("Members: {} GUIDs", details.guids.len());

        for guid_info in details.guids.iter().take(5) {
            println!("  - {} ({})", guid_info.guid, guid_info.membership);
        }
        if details.guids.len() > 5 {
            println!("  ... and {} more", details.guids.len() - 5);
        }
    }

    println!("\nDone!");
    Ok(())
}
