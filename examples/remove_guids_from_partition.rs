//! Example: Remove GUIDs from a partition key.
//!
//! This example demonstrates how to remove port GUIDs from an existing partition key.
//!
//! Run with: cargo run --example remove_guids_from_partition
//!
//! Environment variables:
//! - UFM_HOST: The UFM server hostname (default: localhost)
//! - UFM_USER: The UFM username (default: admin)
//! - UFM_PASSWORD: The UFM password (default: admin)
//! - PKEY: The partition key to remove GUIDs from (default: 0x42)
//! - GUIDS: Comma-separated list of GUIDs to remove (required)

use std::env;

use ufm_client::types::{Guid, Pkey};
use ufm_client::{Auth, UfmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment
    let host = env::var("UFM_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("UFM_USER").unwrap_or_else(|_| "admin".to_string());
    let pass = env::var("UFM_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    let pkey_str = env::var("PKEY").unwrap_or_else(|_| "0x42".to_string());

    // Parse GUIDs from environment
    let guids_str = match env::var("GUIDS") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: GUIDS environment variable is required.");
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  GUIDS=guid1,guid2 cargo run --example remove_guids_from_partition");
            eprintln!();
            eprintln!("Example:");
            eprintln!(
                "  GUIDS=0002c903000e0b72,0002c903000e0b73 PKEY=0x42 cargo run --example remove_guids_from_partition"
            );
            eprintln!();
            eprintln!("To see current partition members, run: PKEY=0x42 cargo run --example view_partition");
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

    println!("\n=== Removing GUIDs from Partition {pkey} ===\n");

    // Get current partition details
    let details = match client.pkeys().get(&pkey).await {
        Ok(d) => d,
        Err(e) if e.is_not_found() => {
            eprintln!("Partition {pkey} does not exist.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    println!("Current partition has {} members", details.guids.len());
    println!("\nGUIDs to remove:");
    for guid in &guids {
        let is_member = details.guids.iter().any(|g| &g.guid == guid);
        let status = if is_member { "member" } else { "not a member" };
        println!("  {guid} ({status})");
    }

    // Remove the GUIDs from the partition
    client.pkeys().remove_guids(&pkey, &guids).await?;

    println!(
        "\nSuccessfully removed {} GUID(s) from partition {pkey}",
        guids.len()
    );

    // Show updated partition details
    let updated_details = client.pkeys().get(&pkey).await?;
    println!("Partition now has {} members", updated_details.guids.len());

    Ok(())
}
