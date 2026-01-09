//! Virtual Ports (VPorts) API for virtualization support.
//!
//! This module provides operations for querying virtual ports when
//! virtualization is enabled on the UFM subnet manager.
//!
//! # Virtualization Check
//!
//! The vports endpoint can be used to check if virtualization is enabled.
//! When virtualization is not enabled, the endpoint returns an empty list
//! or an error.
//!
//! # Examples
//!
//! ```no_run
//! use ufm_client::{UfmClient, Auth};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = UfmClient::builder()
//!     .base_url("https://ufm-server")
//!     .auth(Auth::basic("admin", "password"))
//!     .build()?;
//!
//! // Check if virtualization is enabled
//! if client.vports().is_virtualization_enabled().await? {
//!     println!("Virtualization is enabled!");
//!     
//!     // List all vports
//!     let vports = client.vports().list().await?;
//!     for vport in vports {
//!         println!("VPort {} on {}", vport.virtual_port_guid, vport.system_name);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use tracing::debug;

use crate::client::UfmClient;
use crate::error::{Error, Result};
use crate::types::Vport;

/// API path for vports endpoint.
const VPORTS_PATH: &str = "/resources/vports";

/// The VPorts API handle.
///
/// Provides methods for querying virtual ports and checking virtualization status.
pub struct VportsApi<'a> {
    client: &'a UfmClient,
}

impl<'a> VportsApi<'a> {
    /// Create a new VPorts API handle.
    pub(crate) fn new(client: &'a UfmClient) -> Self {
        Self { client }
    }

    /// List all virtual ports.
    ///
    /// Returns an empty list if virtualization is not enabled or there are no vports.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or authentication fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::{UfmClient, Auth};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = UfmClient::builder()
    ///     .base_url("https://ufm-server")
    ///     .auth(Auth::basic("admin", "password"))
    ///     .build()?;
    ///
    /// let vports = client.vports().list().await?;
    /// for vport in vports {
    ///     println!("{}: {} ({})",
    ///         vport.virtual_port_guid,
    ///         vport.system_name,
    ///         vport.state);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<Vec<Vport>> {
        debug!("Listing all vports");
        self.client.get(VPORTS_PATH).await
    }

    /// Check if virtualization is enabled on the subnet manager.
    ///
    /// This queries the vports endpoint and returns true if:
    /// - The endpoint responds successfully (even with an empty list)
    ///
    /// Returns false if the endpoint returns certain error codes indicating
    /// virtualization is not enabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::{UfmClient, Auth};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = UfmClient::builder()
    ///     .base_url("https://ufm-server")
    ///     .auth(Auth::basic("admin", "password"))
    ///     .build()?;
    ///
    /// if client.vports().is_virtualization_enabled().await? {
    ///     println!("Virtualization is enabled!");
    /// } else {
    ///     println!("Virtualization is not enabled.");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_virtualization_enabled(&self) -> Result<bool> {
        debug!("Checking if virtualization is enabled");
        match self.list().await {
            Ok(_) => Ok(true),
            Err(Error::Api {
                status_code: 400,
                ref message,
            }) if message.contains("virtualization") || message.contains("Virtualization") => {
                Ok(false)
            }
            Err(Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get vports filtered by system GUID.
    ///
    /// Returns only the vports associated with the specified physical system.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::{UfmClient, Auth};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = UfmClient::builder()
    ///     .base_url("https://ufm-server")
    ///     .auth(Auth::basic("admin", "password"))
    ///     .build()?;
    ///
    /// let vports = client.vports().by_system_guid("248a0703008a850a").await?;
    /// println!("Found {} vports for system", vports.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn by_system_guid(&self, system_guid: &str) -> Result<Vec<Vport>> {
        debug!("Fetching vports for system_guid={}", system_guid);
        let all_vports = self.list().await?;
        let filtered: Vec<Vport> = all_vports
            .into_iter()
            .filter(|v| v.system_guid == system_guid)
            .collect();
        Ok(filtered)
    }
}
