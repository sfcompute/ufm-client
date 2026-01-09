//! Example: Add GUIDs to a partition key.
//!
//! This example demonstrates how to add port GUIDs to an existing partition key
//! with configurable membership levels.
//!
//! Run with: cargo run --example add_guids_to_partition
//!
//! Environment variables:
//! - UFM_HOST: The UFM server hostname (default: localhost)
//! - UFM_USER: The UFM username (default: admin)
//! - UFM_PASSWORD: The UFM password (default: admin)
//! - PKEY: The partition key to add GUIDs to (default: 0x42)
//! - GUIDS: Comma-separated list of GUIDs to add (required)
//! - MEMBERSHIP: Membership type - "full" or "limited" (default: full)

use std::env;

use ufm_client::types::{Guid, Membership, Pkey};
use ufm_client::{Auth, UfmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment
    let host = env::var("UFM_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("UFM_USER").unwrap_or_else(|_| "admin".to_string());
    let pass = env::var("UFM_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    let pkey_str = env::var("PKEY").unwrap_or_else(|_| "0x42".to_string());
    let membership_str = env::var("MEMBERSHIP").unwrap_or_else(|_| "full".to_string());

    // Parse GUIDs from environment
    let guids_str = match env::var("GUIDS") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: GUIDS environment variable is required.");
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  GUIDS=guid1,guid2 cargo run --example add_guids_to_partition");
            eprintln!();
            eprintln!("Example:");
            eprintln!(
                "  GUIDS=0002c903000e0b72,0002c903000e0b73 PKEY=0x42 cargo run --example add_guids_to_partition"
            );
            eprintln!();
            eprintln!("To find available GUIDs, run: cargo run --example list_guids");
            return Ok(());
        }
    };

    let base_url = format!("https://{host}");
    println!("Connecting to UFM at {base_url}...");

    // Create the client
    let client = UfmClient::builder()
        .base_url(&base_url)
        .auth(Auth::basic(&user, &pass))
        .verify_ssl(false)
        .build()?;

    // Parse inputs
    let pkey: Pkey = pkey_str.parse()?;
    let membership: Membership = membership_str.parse()?;

    // Parse GUIDs from comma-separated string
    let guids: Vec<Guid> = guids_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()?;

    if guids.is_empty() {
        eprintln!("Error: No valid GUIDs provided.");
        return Ok(());
    }

    println!("\n=== Adding GUIDs to Partition {pkey} ===\n");
    println!("GUIDs to add:");
    for guid in &guids {
        println!("  {guid}");
    }
    println!("Membership: {membership}");

    // Verify the partition exists
    if client.pkeys().get(&pkey).await.is_err() {
        eprintln!("\nPartition {pkey} does not exist.");
        eprintln!("Create it first with: PKEY={pkey} cargo run --example create_partition");
        return Ok(());
    }

    // Add the GUIDs to the partition
    client
        .pkeys()
        .add_guids(&pkey, &guids)
        .membership(membership)
        .ip_over_ib(true)
        .send()
        .await?;

    println!(
        "\nSuccessfully added {} GUID(s) to partition {pkey}",
        guids.len()
    );

    // Show updated partition details
    let details = client.pkeys().get(&pkey).await?;
    println!("\nUpdated partition has {} members", details.guids.len());

    Ok(())
}
