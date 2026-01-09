//! Strongly-typed domain models for UFM resources.
//!
//! This module provides type-safe representations of UFM concepts
//! including GUIDs, PKeys, membership types, QoS configurations, and VPorts.

mod guid;
mod membership;
mod mtu;
mod partition;
mod pkey;
mod qos;
mod service_level;
mod vport;

pub use guid::Guid;
pub use membership::Membership;
pub use mtu::{MtuLimit, MtuLimitError};
pub use partition::{GuidInfo, Partition, PartitionDetails, PartitionsMap};
pub use pkey::Pkey;
pub use qos::{QosConfig, QosConfigBuilder};
pub use service_level::{ServiceLevel, ServiceLevelError};
pub use vport::{Vport, VportState};
