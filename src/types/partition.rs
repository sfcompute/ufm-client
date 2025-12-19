//! Partition types for UFM partition (PKey) management.
//!
//! This module contains types for representing partition information
//! including member GUIDs and their properties.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{Guid, Membership, Pkey, QosConfig};

/// Information about a GUID's membership in a partition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuidInfo {
    /// The GUID of the port
    pub guid: Guid,
    /// The membership level
    #[serde(default)]
    pub membership: Membership,
    /// Whether this is an index0 port
    #[serde(default)]
    pub index0: bool,
}

impl GuidInfo {
    /// Create a new GuidInfo with the given GUID and membership.
    pub fn new(guid: Guid, membership: Membership) -> Self {
        Self {
            guid,
            membership,
            index0: false,
        }
    }

    /// Create a new GuidInfo with index0 flag.
    pub fn with_index0(mut self, index0: bool) -> Self {
        self.index0 = index0;
        self
    }
}

/// Basic partition information (used in list responses).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Partition {
    /// The partition name
    pub partition: String,
    /// Whether IP over IB is enabled
    #[serde(default)]
    pub ip_over_ib: bool,
}

/// Detailed partition information including member GUIDs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartitionDetails {
    /// The partition name
    pub partition: String,
    /// Whether IP over IB is enabled
    #[serde(default)]
    pub ip_over_ib: bool,
    /// List of member GUIDs with their membership info
    #[serde(default)]
    pub guids: Vec<GuidInfo>,
    /// QoS configuration (if present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qos_conf: Option<QosConfig>,
}

impl PartitionDetails {
    /// Get all GUIDs in this partition.
    pub fn guid_list(&self) -> Vec<&Guid> {
        self.guids.iter().map(|g| &g.guid).collect()
    }

    /// Get all full members.
    pub fn full_members(&self) -> Vec<&GuidInfo> {
        self.guids
            .iter()
            .filter(|g| g.membership.is_full())
            .collect()
    }

    /// Get all limited members.
    pub fn limited_members(&self) -> Vec<&GuidInfo> {
        self.guids
            .iter()
            .filter(|g| g.membership.is_limited())
            .collect()
    }
}

/// A map of PKeys to their partition details (used in get all response).
pub type PartitionsMap = HashMap<Pkey, PartitionDetails>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guid_info_new() {
        let guid = Guid::new("0002c903000e0b72").unwrap();
        let info = GuidInfo::new(guid.clone(), Membership::Full);
        assert_eq!(info.guid, guid);
        assert!(info.membership.is_full());
        assert!(!info.index0);
    }

    #[test]
    fn test_partition_details_members() {
        let details = PartitionDetails {
            partition: "test_pkey".to_string(),
            ip_over_ib: true,
            guids: vec![
                GuidInfo::new(Guid::new("0002c903000e0b72").unwrap(), Membership::Full),
                GuidInfo::new(Guid::new("0002c903000e0b73").unwrap(), Membership::Limited),
            ],
            qos_conf: None,
        };

        assert_eq!(details.full_members().len(), 1);
        assert_eq!(details.limited_members().len(), 1);
        assert_eq!(details.guid_list().len(), 2);
    }
}
