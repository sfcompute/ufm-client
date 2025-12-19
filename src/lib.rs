//! UFM Client - A Rust client library for NVIDIA's Unified Fabric Manager REST API.
//!
//! This library provides a type-safe, async interface for interacting with UFM
//! to manage InfiniBand fabrics, partition keys (PKeys), and port memberships.
//!
//! # Quick Start
//!
//! ```no_run
//! use ufm_client::{UfmClient, Auth};
//! use ufm_client::types::{Pkey, Guid, Membership};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client with basic authentication
//! let client = UfmClient::builder()
//!     .base_url("https://ufm-server")
//!     .auth(Auth::basic("admin", "password"))
//!     .verify_ssl(false) // For self-signed certificates
//!     .build()?;
//!
//! // Get UFM version
//! let version = client.version().await?;
//! println!("Connected to UFM {}", version.ufm_release_version);
//!
//! // List all partition keys
//! let pkeys = client.pkeys().list().await?;
//!
//! // Create a new partition key
//! let pkey: Pkey = "0x12".parse()?;
//! client.pkeys()
//!     .create(&pkey)
//!     .ip_over_ib(true)
//!     .send()
//!     .await?;
//!
//! // Add GUIDs to the partition
//! let guid1: Guid = "0002c903000e0b72".parse()?;
//! let guid2: Guid = "0002c903000e0b73".parse()?;
//! client.pkeys()
//!     .add_guids(&pkey, &[guid1, guid2])
//!     .membership(Membership::Full)
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **Type-safe API**: Strongly-typed domain models prevent common errors
//! - **Async/await**: Built on tokio and reqwest for efficient async I/O
//! - **Builder pattern**: Fluent API for complex operations
//! - **Pluggable auth**: Support for Basic, Token, and Session authentication
//! - **Retry logic**: Automatic retries for transient failures
//! - **Comprehensive errors**: Detailed error types for proper error handling

pub mod api;
pub mod auth;
pub mod client;
pub mod error;
pub mod http;
pub mod types;

// Re-export main types at crate root for convenience
pub use auth::Auth;
pub use client::{UfmClient, UfmClientBuilder};
pub use error::{Error, Result};

/// Prelude module for convenient imports.
///
/// ```
/// use ufm_client::prelude::*;
/// ```
pub mod prelude {
    pub use crate::auth::Auth;
    pub use crate::client::UfmClient;
    pub use crate::error::{Error, Result};
    pub use crate::types::{
        Guid, Membership, Partition, PartitionDetails, Pkey, QosConfig, Vport, VportState,
    };
}
