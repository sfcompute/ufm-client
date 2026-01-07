//! Quality of Service (QoS) configuration for partitions.
//!
//! QoS settings control bandwidth, MTU, and service level for partition traffic.

use serde::{Deserialize, Serialize};

use super::{MtuLimit, ServiceLevel};

/// Quality of Service configuration for a partition.
///
/// # Examples
///
/// ```
/// use ufm_client::types::{MtuLimit, QosConfig, ServiceLevel};
///
/// let qos = QosConfig::builder()
///     .mtu_limit(MtuLimit::Mtu2048)
///     .rate_limit(300.0)
///     .service_level(ServiceLevel::SL3)
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QosConfig {
    /// MTU limit (256, 512, 1024, 2048, or 4096 bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu_limit: Option<MtuLimit>,

    /// Rate limit in Gbps (e.g., 2.5, 300.0)
    #[serde(alias = "Rate_limit", skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<f64>,

    /// Service level (0-15)
    #[serde(alias = "Service_level", skip_serializing_if = "Option::is_none")]
    pub service_level: Option<ServiceLevel>,
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
    mtu_limit: Option<MtuLimit>,
    rate_limit: Option<f64>,
    service_level: Option<ServiceLevel>,
}

impl QosConfigBuilder {
    /// Set the MTU limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use ufm_client::types::{MtuLimit, QosConfig};
    ///
    /// let qos = QosConfig::builder()
    ///     .mtu_limit(MtuLimit::Mtu2048)
    ///     .build();
    /// ```
    pub fn mtu_limit(mut self, value: MtuLimit) -> Self {
        self.mtu_limit = Some(value);
        self
    }

    /// Set the rate limit in Gbps.
    pub fn rate_limit(mut self, value: f64) -> Self {
        self.rate_limit = Some(value);
        self
    }

    /// Set the service level.
    ///
    /// # Examples
    ///
    /// ```
    /// use ufm_client::types::{QosConfig, ServiceLevel};
    ///
    /// let qos = QosConfig::builder()
    ///     .service_level(ServiceLevel::SL3)
    ///     .build();
    /// ```
    pub fn service_level(mut self, value: ServiceLevel) -> Self {
        self.service_level = Some(value);
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
            .mtu_limit(MtuLimit::Mtu2048)
            .rate_limit(300.0)
            .service_level(ServiceLevel::SL3)
            .build();

        assert_eq!(qos.mtu_limit, Some(MtuLimit::Mtu2048));
        assert_eq!(qos.rate_limit, Some(300.0));
        assert_eq!(qos.service_level, Some(ServiceLevel::SL3));
    }

    #[test]
    fn test_qos_default() {
        let qos = QosConfig::default();
        assert!(qos.is_empty());
    }

    #[test]
    fn test_qos_serialization() {
        let qos = QosConfig::builder()
            .mtu_limit(MtuLimit::Mtu2048)
            .service_level(ServiceLevel::SL5)
            .rate_limit(100.0)
            .build();

        let json = serde_json::to_value(&qos).unwrap();
        // Verify we serialize to numeric values for API compatibility
        assert_eq!(json["mtu_limit"], 4);
        assert_eq!(json["service_level"], 5);
        assert_eq!(json["rate_limit"], 100.0);
    }

    #[test]
    fn test_qos_deserialization() {
        let json = r#"{"mtu_limit": 3, "service_level": 7, "rate_limit": 50.0}"#;
        let qos: QosConfig = serde_json::from_str(json).unwrap();

        assert_eq!(qos.mtu_limit, Some(MtuLimit::Mtu1024));
        assert_eq!(qos.service_level, Some(ServiceLevel::SL7));
        assert_eq!(qos.rate_limit, Some(50.0));
    }
}
