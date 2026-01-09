//! Example: View partition key details.
//!
//! This example demonstrates how to view the details of a specific partition key,
//! including its members and QoS configuration.
//!
//! Run with: cargo run --example view_partition
//!
//! Environment variables:
//! - UFM_HOST: The UFM server hostname (default: localhost)
//! - UFM_USER: The UFM username (default: admin)
//! - UFM_PASSWORD: The UFM password (default: admin)
//! - PKEY: The partition key to view (default: 0x7fff - the default partition)

use std::env;

use ufm_client::types::Pkey;
use ufm_client::{Auth, UfmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment
    let host = env::var("UFM_HOST").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("UFM_USER").unwrap_or_else(|_| "admin".to_string());
    let pass = env::var("UFM_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    let pkey_str = env::var("PKEY").unwrap_or_else(|_| "0x7fff".to_string());

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
    println!("\n=== Partition Key {pkey} Details ===\n");

    // Get partition details
    match client.pkeys().get(&pkey).await {
        Ok(details) => {
            // Basic information
            println!("Partition name: {}", details.partition);
            println!("IP over IB: {}", details.ip_over_ib);
            println!("Total members: {} GUIDs", details.guids.len());

            // QoS configuration
            if let Some(qos) = &details.qos_conf {
                println!("\nQoS Configuration:");
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

            // Member breakdown
            let full_members = details.full_members();
            let limited_members = details.limited_members();

            println!("\nMembership breakdown:");
            println!("  Full members: {}", full_members.len());
            println!("  Limited members: {}", limited_members.len());

            // Show first few members
            println!("\nMembers (first 10):");
            for guid_info in details.guids.iter().take(10) {
                let membership = if guid_info.membership.is_full() {
                    "full"
                } else {
                    "limited"
                };
                let index0 = if guid_info.index0 { " [index0]" } else { "" };
                println!("  {} ({membership}){index0}", guid_info.guid);
            }

            if details.guids.len() > 10 {
                println!("  ... and {} more", details.guids.len() - 10);
            }
        }
        Err(e) if e.is_not_found() => {
            println!("Partition key {pkey} not found.");
            println!("\nAvailable partition keys:");
            let pkeys = client.pkeys().list().await?;
            for pk in pkeys {
                println!("  {pk}");
            }
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    Ok(())
}
