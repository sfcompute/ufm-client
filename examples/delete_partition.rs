//! Example: Delete a partition key.
//!
//! This example demonstrates how to delete a partition key from UFM.
//!
//! **Warning**: This is a destructive operation that cannot be undone!
//!
//! Run with: cargo run --example delete_partition
//!
//! Environment variables:
//! - UFM_HOST: The UFM server hostname (default: localhost)
//! - UFM_USER: The UFM username (default: admin)
//! - UFM_PASSWORD: The UFM password (default: admin)
//! - PKEY: The partition key to delete (required)
//! - CONFIRM: Set to "yes" to skip confirmation prompt (optional)

use std::env;
use std::io::{self, Write};

use ufm_client::types::Pkey;
use ufm_client::{Auth, UfmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment
    let host = env::var("UFM_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("UFM_USER").unwrap_or_else(|_| "admin".to_string());
    let pass = env::var("UFM_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    let confirm = env::var("CONFIRM").unwrap_or_default();

    // Require PKEY to be explicitly set for safety
    let pkey_str = match env::var("PKEY") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: PKEY environment variable is required.");
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  PKEY=0x42 cargo run --example delete_partition");
            eprintln!();
            eprintln!("To skip confirmation:");
            eprintln!("  PKEY=0x42 CONFIRM=yes cargo run --example delete_partition");
            eprintln!();
            eprintln!("Available partition keys:");

            // Show available pkeys
            let base_url = format!("https://{host}");
            let client = UfmClient::builder()
                .base_url(&base_url)
                .auth(Auth::basic(&user, &pass))
                .verify_ssl(false)
                .build()?;

            let pkeys = client.pkeys().list().await?;
            for pk in pkeys {
                // Skip the default partition
                if pk.as_hex_string() == "0x7fff" {
                    println!("  {pk} (default - cannot be deleted)");
                } else {
                    println!("  {pk}");
                }
            }

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

    // Parse the PKey
    let pkey: Pkey = pkey_str.parse()?;

    // Prevent deletion of the default partition
    if pkey.as_hex_string() == "0x7fff" {
        eprintln!("Error: Cannot delete the default management partition (0x7fff).");
        return Ok(());
    }

    println!("\n=== Delete Partition {pkey} ===\n");

    // Get partition details first
    let details = match client.pkeys().get(&pkey).await {
        Ok(d) => d,
        Err(e) if e.is_not_found() => {
            println!("Partition {pkey} does not exist.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    println!("Partition: {}", details.partition);
    println!("Members: {} GUIDs", details.guids.len());
    println!("IP over IB: {}", details.ip_over_ib);

    // Confirmation
    if confirm.to_lowercase() != "yes" {
        print!("\nAre you sure you want to delete this partition? (yes/no): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Delete the partition
    client.pkeys().delete(&pkey).await?;

    println!("\nSuccessfully deleted partition {pkey}");

    Ok(())
}
