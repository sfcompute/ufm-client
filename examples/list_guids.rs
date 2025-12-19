//! Example: List all GUIDs in the fabric.
//!
//! This example demonstrates how to list all GUIDs in an InfiniBand fabric
//! by querying the default management partition (0x7fff), which all fabric
//! members belong to.
//!
//! Run with: cargo run --example list_guids
//!
//! Environment variables:
//! - UFM_HOST: The UFM server hostname (default: localhost)
//! - UFM_USER: The UFM username (default: admin)
//! - UFM_PASSWORD: The UFM password (default: admin)

use std::env;

use ufm_client::types::Pkey;
use ufm_client::{Auth, UfmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // The default management partition (0x7fff) contains all fabric members.
    // By querying this partition, we can get all GUIDs in the fabric.
    let default_pkey: Pkey = "0x7fff".parse()?;

    println!("\n=== All GUIDs in Fabric ===");
    println!("Querying default management partition ({default_pkey})...\n");

    match client.pkeys().get(&default_pkey).await {
        Ok(details) => {
            println!("Found {} GUIDs in the fabric:\n", details.guids.len());

            // Print all GUIDs with their membership type
            for guid_info in &details.guids {
                let membership = if guid_info.membership.is_full() {
                    "full"
                } else {
                    "limited"
                };
                let index0 = if guid_info.index0 { " (index0)" } else { "" };
                println!("  {}: {}{}", guid_info.guid, membership, index0);
            }

            // Summary
            println!("\n--- Summary ---");
            println!("Total GUIDs: {}", details.guids.len());
            println!("Full members: {}", details.full_members().len());
            println!("Limited members: {}", details.limited_members().len());
        }
        Err(e) => {
            eprintln!("Failed to get fabric GUIDs: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}
