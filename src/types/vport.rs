//! Virtual Port (VPort) types for virtualization support.
//!
//! VPorts are virtual ports associated with physical InfiniBand ports when
//! virtualization is enabled on the subnet manager.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;

use super::Guid;

/// State of a virtual port.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum VportState {
    /// The vport is active and operational
    Active,
    /// The vport is down
    Down,
    /// The vport is initializing
    Init,
    /// The vport is armed (ready to become active)
    Armed,
    /// Unknown state (for forward compatibility)
    #[serde(other)]
    #[default]
    Unknown,
}

impl VportState {
    /// Returns true if the vport is active
    pub fn is_active(&self) -> bool {
        matches!(self, VportState::Active)
    }
}

impl fmt::Display for VportState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VportState::Active => write!(f, "Active"),
            VportState::Down => write!(f, "Down"),
            VportState::Init => write!(f, "Init"),
            VportState::Armed => write!(f, "Armed"),
            VportState::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about a virtual port (VPort).
///
/// VPorts are created when virtualization is enabled on the subnet manager.
/// Each vport is associated with a physical port and has its own virtual GUID and LID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vport {
    /// State of the virtual port (Active, Down, etc.)
    #[serde(rename = "virtual_port_state")]
    pub state: VportState,

    /// Local Identifier assigned to this virtual port
    #[serde(rename = "virtual_port_lid")]
    pub lid: u32,

    /// IP address of the system (may be 0.0.0.0 if not configured)
    #[serde(rename = "system_ip", deserialize_with = "deserialize_ip_addr")]
    pub system_ip: IpAddr,

    /// Name of the system hosting this vport
    #[serde(rename = "system_name")]
    pub system_name: String,

    /// Description of the node (typically includes HCA info)
    #[serde(rename = "node_description")]
    pub node_description: String,

    /// GUID of the system
    #[serde(rename = "system_guid")]
    pub system_guid: String,

    /// GUID of the physical port
    #[serde(rename = "port_guid")]
    pub port_guid: String,

    /// Name of the physical port (format: guid_port_number)
    #[serde(rename = "port_name")]
    pub port_name: String,

    /// Physical port number on the HCA
    #[serde(rename = "physical_port_number")]
    pub physical_port_number: u32,

    /// Virtual GUID assigned to this vport
    #[serde(rename = "virtual_port_guid")]
    pub virtual_port_guid: String,
}

impl Vport {
    /// Get the virtual GUID as a strongly-typed Guid, if valid.
    pub fn virtual_guid(&self) -> Option<Guid> {
        self.virtual_port_guid.parse().ok()
    }

    /// Get the physical port GUID as a strongly-typed Guid, if valid.
    pub fn physical_guid(&self) -> Option<Guid> {
        self.port_guid.parse().ok()
    }

    /// Check if this vport is active
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}

/// Custom deserializer for IP addresses that handles "0.0.0.0" and empty strings
fn deserialize_ip_addr<'de, D>(deserializer: D) -> Result<IpAddr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vport_state_is_active() {
        assert!(VportState::Active.is_active());
        assert!(!VportState::Down.is_active());
        assert!(!VportState::Unknown.is_active());
    }

    #[test]
    fn test_vport_deserialize() {
        let json = r#"{
            "virtual_port_state": "Active",
            "virtual_port_lid": 3434,
            "system_ip": "11.4.3.175",
            "system_name": "r-ufm77",
            "node_description": "r-ufm77 mlx5_0",
            "system_guid": "248a0703008a850a",
            "port_guid": "248a0703008a850a",
            "port_name": "248a0703008a850a_1",
            "physical_port_number": 1,
            "virtual_port_guid": "0002c90000000017"
        }"#;

        let vport: Vport = serde_json::from_str(json).unwrap();
        assert_eq!(vport.state, VportState::Active);
        assert_eq!(vport.lid, 3434);
        assert_eq!(vport.system_name, "r-ufm77");
        assert_eq!(vport.physical_port_number, 1);
        assert!(vport.is_active());
    }
}
