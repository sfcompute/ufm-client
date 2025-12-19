//! Quality of Service (QoS) configuration for partitions.
//!
//! QoS settings control bandwidth, MTU, and service level for partition traffic.

use serde::{Deserialize, Serialize};

/// Quality of Service configuration for a partition.
///
/// # Examples
///
/// ```
/// use ufm_client::types::QosConfig;
///
/// let qos = QosConfig::builder()
///     .mtu_limit(4)
///     .rate_limit(300.0)
///     .service_level(3)
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QosConfig {
    /// MTU limit (1-5, where 5 = 4096 bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu_limit: Option<u8>,

    /// Rate limit in Gbps (e.g., 2.5, 300.0)
    #[serde(alias = "Rate_limit", skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<f64>,

    /// Service level (0-15)
    #[serde(alias = "Service_level", skip_serializing_if = "Option::is_none")]
    pub service_level: Option<u8>,
}

impl QosConfig {
    /// Create a new QoS configuration builder.
    pub fn builder() -> QosConfigBuilder {
        QosConfigBuilder::default()
    }

    /// Check if all QoS values are None.
    pub fn is_empty(&self) -> bool {
        self.mtu_limit.is_none() && self.rate_limit.is_none() && self.service_level.is_none()
    }
}

/// Builder for QoS configuration.
#[derive(Debug, Default)]
pub struct QosConfigBuilder {
    mtu_limit: Option<u8>,
    rate_limit: Option<f64>,
    service_level: Option<u8>,
}

impl QosConfigBuilder {
    /// Set the MTU limit (1-5).
    ///
    /// MTU values:
    /// - 1 = 256 bytes
    /// - 2 = 512 bytes
    /// - 3 = 1024 bytes
    /// - 4 = 2048 bytes
    /// - 5 = 4096 bytes
    pub fn mtu_limit(mut self, value: u8) -> Self {
        self.mtu_limit = Some(value.clamp(1, 5));
        self
    }

    /// Set the rate limit in Gbps.
    pub fn rate_limit(mut self, value: f64) -> Self {
        self.rate_limit = Some(value);
        self
    }

    /// Set the service level (0-15).
    pub fn service_level(mut self, value: u8) -> Self {
        self.service_level = Some(value.clamp(0, 15));
        self
    }

    /// Build the QoS configuration.
    pub fn build(self) -> QosConfig {
        QosConfig {
            mtu_limit: self.mtu_limit,
            rate_limit: self.rate_limit,
            service_level: self.service_level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_builder() {
        let qos = QosConfig::builder()
            .mtu_limit(4)
            .rate_limit(300.0)
            .service_level(3)
            .build();

        assert_eq!(qos.mtu_limit, Some(4));
        assert_eq!(qos.rate_limit, Some(300.0));
        assert_eq!(qos.service_level, Some(3));
    }

    #[test]
    fn test_qos_default() {
        let qos = QosConfig::default();
        assert!(qos.is_empty());
    }

    #[test]
    fn test_qos_mtu_clamped() {
        let qos = QosConfig::builder().mtu_limit(10).build();
        assert_eq!(qos.mtu_limit, Some(5));
    }

    #[test]
    fn test_qos_service_level_clamped() {
        let qos = QosConfig::builder().service_level(20).build();
        assert_eq!(qos.service_level, Some(15));
    }
}
