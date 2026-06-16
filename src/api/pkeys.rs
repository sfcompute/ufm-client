//! Partition Key (PKey) management API.
//!
//! This module provides operations for managing InfiniBand partition keys (PKeys)
//! including listing, creating, modifying, and deleting partitions.
//!
//! # Examples
//!
//! ```no_run
//! use ufm_client::{UfmClient, Auth};
//! use ufm_client::types::{Pkey, Guid, Membership};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = UfmClient::builder()
//!     .base_url("https://ufm-server")
//!     .auth(Auth::basic("admin", "password"))
//!     .build()?;
//!
//! // List all PKeys
//! let pkeys = client.pkeys().list().await?;
//!
//! // Get details of a specific PKey
//! let pkey: Pkey = "0x12".parse()?;
//! let details = client.pkeys().get(&pkey).await?;
//!
//! // Add GUIDs to a PKey with builder pattern
//! let guid1: Guid = "0002c903000e0b72".parse()?;
//! let guid2: Guid = "0002c903000e0b73".parse()?;
//! client.pkeys()
//!     .add_guids(&pkey, &[guid1, guid2])
//!     .membership(Membership::Full)
//!     .ip_over_ib(true)
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use serde::Deserialize;
use serde_json::json;

use crate::client::UfmClient;
use crate::error::{Error, Result};
use crate::types::{Guid, Membership, MtuLimit, PartitionDetails, Pkey, QosConfig, ServiceLevel};

/// API path for pkeys endpoint.
const PKEYS_PATH: &str = "/resources/pkeys";

/// The PKeys API handle.
///
/// Provides methods for managing partition keys.
#[derive(Debug)]
pub struct PkeysApi<'a> {
    client: &'a UfmClient,
}

impl<'a> PkeysApi<'a> {
    /// Create a new PKeys API handle.
    pub(crate) fn new(client: &'a UfmClient) -> Self {
        Self { client }
    }

    /// List all partition keys.
    ///
    /// Returns a list of PKey hex strings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkeys = client.pkeys().list().await?;
    /// for pkey in pkeys {
    ///     println!("PKey: {}", pkey);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<Vec<Pkey>> {
        self.client.get(PKEYS_PATH).await
    }

    /// Get all partition keys with their full details.
    ///
    /// Returns a map of PKey to partition details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let all = client.pkeys().get_all().await?;
    /// for (pkey, details) in all {
    ///     println!("PKey {}: {} members", pkey, details.guids.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all(&self) -> Result<std::collections::HashMap<Pkey, PartitionDetails>> {
        let path = format!("{PKEYS_PATH}?guids_data=true&qos_conf=true");
        self.client.get(&path).await
    }

    /// Get details of a specific partition key.
    ///
    /// # Arguments
    ///
    /// * `pkey` - The partition key to retrieve
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # use ufm_client::types::Pkey;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkey: Pkey = "0x12".parse()?;
    /// let details = client.pkeys().get(&pkey).await?;
    /// println!("Partition: {}", details.partition);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, pkey: &Pkey) -> Result<PartitionDetails> {
        let path = format!("{PKEYS_PATH}/{pkey}");
        // UFM answers a nonexistent pkey with `200 {}` rather than 404, which won't deserialize
        // into PartitionDetails. Parse leniently and treat a body with no `partition` as not found.
        let value: serde_json::Value = self.client.get(&path).await?;
        if value.get("partition").is_none() {
            return Err(Error::not_found(format!("pkey {pkey}")));
        }
        serde_json::from_value(value).map_err(Error::from)
    }

    /// Create a new empty partition key.
    ///
    /// Returns a builder for configuring the new partition.
    ///
    /// # Arguments
    ///
    /// * `pkey` - The partition key value to create
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # use ufm_client::types::Pkey;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkey: Pkey = "0x12".parse()?;
    /// client.pkeys()
    ///     .create(&pkey)
    ///     .ip_over_ib(true)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(&self, pkey: &Pkey) -> CreatePkeyBuilder<'_> {
        CreatePkeyBuilder::new(self.client, pkey.clone())
    }

    /// Add GUIDs to a partition key.
    ///
    /// Returns a builder for configuring the membership.
    ///
    /// # Arguments
    ///
    /// * `pkey` - The partition key to add GUIDs to
    /// * `guids` - The GUIDs to add
    pub fn add_guids<'b>(&self, pkey: &Pkey, guids: &'b [Guid]) -> AddGuidsBuilder<'a, 'b> {
        AddGuidsBuilder::new(self.client, pkey.clone(), guids)
    }

    /// Remove GUIDs from a partition key.
    ///
    /// # Arguments
    ///
    /// * `pkey` - The partition key to remove GUIDs from
    /// * `guids` - The GUIDs to remove
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # use ufm_client::types::{Pkey, Guid};
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkey: Pkey = "0x12".parse()?;
    /// let guid: Guid = "0002c903000e0b72".parse()?;
    /// client.pkeys().remove_guids(&pkey, &[guid]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_guids(&self, pkey: &Pkey, guids: &[Guid]) -> Result<()> {
        let guid_strings: Vec<&str> = guids.iter().map(|g| g.as_str()).collect();
        let body = json!({
            "pkey": pkey.as_hex_string(),
            "guids": guid_strings
        });
        self.client
            .delete_with_body::<serde_json::Value>(PKEYS_PATH, body)
            .await?;
        Ok(())
    }

    /// Delete a partition key entirely.
    ///
    /// # Arguments
    ///
    /// * `pkey` - The partition key to delete
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # use ufm_client::types::Pkey;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkey: Pkey = "0x12".parse()?;
    /// client.pkeys().delete(&pkey).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, pkey: &Pkey) -> Result<()> {
        let path = format!("{PKEYS_PATH}/{pkey}");
        self.client.delete::<serde_json::Value>(&path).await?;
        Ok(())
    }

    /// Update the QoS configuration for a partition key.
    ///
    /// # Arguments
    ///
    /// * `pkey` - The partition key to update
    /// * `qos` - The new QoS configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # use ufm_client::types::{MtuLimit, Pkey, QosConfig, ServiceLevel};
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkey: Pkey = "0x12".parse()?;
    /// let qos = QosConfig::builder()
    ///     .mtu_limit(MtuLimit::Mtu2048)
    ///     .rate_limit(300.0)
    ///     .service_level(ServiceLevel::SL3)
    ///     .build();
    /// client.pkeys().set_qos(&pkey, &qos).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_qos(&self, pkey: &Pkey, qos: &QosConfig) -> Result<()> {
        let body = json!({
            "pkey": pkey.as_hex_string(),
            "mtu_limit": qos.mtu_limit.map(|m| m.as_u8()),
            "service_level": qos.service_level.map(|s| s.as_u8()),
            "rate_limit": qos.rate_limit
        });
        self.client
            .put::<serde_json::Value>(PKEYS_PATH, body)
            .await?;
        Ok(())
    }

    /// Get the last updated timestamp for partition configuration.
    pub async fn last_updated(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct LastUpdatedResponse {
            last_updated: String,
        }
        let path = format!("{PKEYS_PATH}/last_updated");
        let response: LastUpdatedResponse = self.client.get(&path).await?;
        Ok(response.last_updated)
    }
}

// ============================================================================
// Builders
// ============================================================================

/// Builder for creating a new empty partition key.
#[derive(Debug)]
pub struct CreatePkeyBuilder<'a> {
    client: &'a UfmClient,
    pkey: Pkey,
    ip_over_ib: Option<bool>,
    index0: Option<bool>,
    mtu_limit: Option<MtuLimit>,
    service_level: Option<ServiceLevel>,
    rate_limit: Option<f64>,
}

/// Path for creating empty pkeys (V1 API).
const PKEYS_ADD_PATH: &str = "/resources/pkeys/add";

impl<'a> CreatePkeyBuilder<'a> {
    fn new(client: &'a UfmClient, pkey: Pkey) -> Self {
        Self {
            client,
            pkey,
            ip_over_ib: None,
            index0: None,
            mtu_limit: None,
            service_level: None,
            rate_limit: None,
        }
    }

    /// Enable or disable IP over InfiniBand for this partition.
    pub fn ip_over_ib(mut self, enabled: bool) -> Self {
        self.ip_over_ib = Some(enabled);
        self
    }

    /// Set whether ports should be added with index0 by default.
    pub fn index0(mut self, enabled: bool) -> Self {
        self.index0 = Some(enabled);
        self
    }

    /// Set the MTU limit.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # use ufm_client::types::{MtuLimit, Pkey};
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkey: Pkey = "0x12".parse()?;
    /// client.pkeys()
    ///     .create(&pkey)
    ///     .mtu_limit(MtuLimit::Mtu2048)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn mtu_limit(mut self, value: MtuLimit) -> Self {
        self.mtu_limit = Some(value);
        self
    }

    /// Set the service level.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # use ufm_client::types::{Pkey, ServiceLevel};
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let pkey: Pkey = "0x12".parse()?;
    /// client.pkeys()
    ///     .create(&pkey)
    ///     .service_level(ServiceLevel::SL3)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn service_level(mut self, value: ServiceLevel) -> Self {
        self.service_level = Some(value);
        self
    }

    /// Set the rate limit in Gbps.
    pub fn rate_limit(mut self, value: f64) -> Self {
        self.rate_limit = Some(value);
        self
    }

    /// Execute the create operation.
    ///
    /// This uses the V1 API endpoint (`/ufmRest/resources/pkeys/add`) which
    /// supports creating empty partitions without any GUIDs.
    pub async fn send(self) -> Result<()> {
        let mut body = json!({
            "pkey": self.pkey.as_hex_string()
        });

        if let Some(ip_over_ib) = self.ip_over_ib {
            body["ip_over_ib"] = json!(ip_over_ib);
        }
        if let Some(index0) = self.index0 {
            body["index0"] = json!(index0);
        }
        if let Some(mtu) = self.mtu_limit {
            body["mtu_limit"] = json!(mtu.as_u8());
        }
        if let Some(sl) = self.service_level {
            body["service_level"] = json!(sl.as_u8());
        }
        if let Some(rate) = self.rate_limit {
            body["rate_limit"] = json!(rate);
        }

        self.client
            .post::<serde_json::Value>(PKEYS_ADD_PATH, body)
            .await?;
        Ok(())
    }
}

/// Builder for adding GUIDs to a partition key.
#[derive(Debug)]
pub struct AddGuidsBuilder<'a, 'b> {
    client: &'a UfmClient,
    pkey: Pkey,
    guids: &'b [Guid],
    membership: Option<Membership>,
    memberships: Option<Vec<Membership>>,
    ip_over_ib: Option<bool>,
    index0: Option<bool>,
    mtu_limit: Option<MtuLimit>,
    service_level: Option<ServiceLevel>,
    rate_limit: Option<f64>,
}

impl<'a, 'b> AddGuidsBuilder<'a, 'b> {
    fn new(client: &'a UfmClient, pkey: Pkey, guids: &'b [Guid]) -> Self {
        Self {
            client,
            pkey,
            guids,
            membership: None,
            memberships: None,
            ip_over_ib: None,
            index0: None,
            mtu_limit: None,
            service_level: None,
            rate_limit: None,
        }
    }

    /// Set the membership level for all GUIDs.
    pub fn membership(mut self, membership: Membership) -> Self {
        self.membership = Some(membership);
        self.memberships = None;
        self
    }

    /// Set individual membership levels for each GUID.
    ///
    /// The length must match the number of GUIDs.
    pub fn memberships(mut self, memberships: Vec<Membership>) -> Self {
        self.memberships = Some(memberships);
        self.membership = None;
        self
    }

    /// Enable or disable IP over InfiniBand.
    pub fn ip_over_ib(mut self, enabled: bool) -> Self {
        self.ip_over_ib = Some(enabled);
        self
    }

    /// Set whether to use index0 for ports.
    pub fn index0(mut self, enabled: bool) -> Self {
        self.index0 = Some(enabled);
        self
    }

    /// Set the MTU limit.
    pub fn mtu_limit(mut self, value: MtuLimit) -> Self {
        self.mtu_limit = Some(value);
        self
    }

    /// Set the service level.
    pub fn service_level(mut self, value: ServiceLevel) -> Self {
        self.service_level = Some(value);
        self
    }

    /// Set the rate limit in Gbps.
    pub fn rate_limit(mut self, value: f64) -> Self {
        self.rate_limit = Some(value);
        self
    }

    /// Execute the add operation.
    pub async fn send(self) -> Result<()> {
        let guid_strings: Vec<&str> = self.guids.iter().map(|g| g.as_str()).collect();

        let mut body = json!({
            "pkey": self.pkey.as_hex_string(),
            "guids": guid_strings
        });

        // Handle membership - either single value for all or array of individual values
        if let Some(memberships) = self.memberships {
            let membership_strs: Vec<&str> = memberships.iter().map(|m| m.as_str()).collect();
            body["membership"] = json!(membership_strs);
        } else if let Some(membership) = self.membership {
            body["membership"] = json!(membership.as_str());
        }

        if let Some(ip_over_ib) = self.ip_over_ib {
            body["ip_over_ib"] = json!(ip_over_ib);
        }
        if let Some(index0) = self.index0 {
            body["index0"] = json!(index0);
        }
        if let Some(mtu) = self.mtu_limit {
            body["mtu_limit"] = json!(mtu.as_u8());
        }
        if let Some(sl) = self.service_level {
            body["service_level"] = json!(sl.as_u8());
        }
        if let Some(rate) = self.rate_limit {
            body["rate_limit"] = json!(rate);
        }

        self.client
            .post::<serde_json::Value>(PKEYS_PATH, body)
            .await?;
        Ok(())
    }
}
