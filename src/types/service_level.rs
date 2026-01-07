//! Service Level (SL) type for QoS configuration.
//!
//! This module provides a strongly-typed wrapper for InfiniBand service levels
//! used in partition QoS configuration. Service levels range from 0-15.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// InfiniBand Service Level for partition QoS configuration.
///
/// Service levels range from 0 to 15, where different levels can be mapped
/// to different virtual lanes (VLs) for traffic prioritization.
///
/// # Examples
///
/// ```
/// use ufm_client::types::ServiceLevel;
///
/// let sl = ServiceLevel::new(3).unwrap();
/// assert_eq!(sl.as_u8(), 3);
///
/// // Invalid values return an error
/// assert!(ServiceLevel::new(16).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceLevel(u8);

impl ServiceLevel {
    /// The minimum valid service level (0).
    pub const MIN: u8 = 0;

    /// The maximum valid service level (15).
    pub const MAX: u8 = 15;

    /// Create a new service level with validation.
    ///
    /// Returns `None` if the value is outside the valid range (0-15).
    pub const fn new(value: u8) -> Result<Self, ServiceLevelError> {
        if value <= Self::MAX {
            Ok(ServiceLevel(value))
        } else {
            Err(ServiceLevelError(value))
        }
    }

    /// Create a service level without validation.
    ///
    /// # Safety
    ///
    /// The value must be in the range 0-15.
    #[inline]
    pub const unsafe fn new_unchecked(value: u8) -> Self {
        ServiceLevel(value)
    }

    /// Get the service level value.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Service level 0 (default/best effort).
    pub const SL0: ServiceLevel = ServiceLevel(0);
    /// Service level 1.
    pub const SL1: ServiceLevel = ServiceLevel(1);
    /// Service level 2.
    pub const SL2: ServiceLevel = ServiceLevel(2);
    /// Service level 3.
    pub const SL3: ServiceLevel = ServiceLevel(3);
    /// Service level 4.
    pub const SL4: ServiceLevel = ServiceLevel(4);
    /// Service level 5.
    pub const SL5: ServiceLevel = ServiceLevel(5);
    /// Service level 6.
    pub const SL6: ServiceLevel = ServiceLevel(6);
    /// Service level 7.
    pub const SL7: ServiceLevel = ServiceLevel(7);
    /// Service level 8.
    pub const SL8: ServiceLevel = ServiceLevel(8);
    /// Service level 9.
    pub const SL9: ServiceLevel = ServiceLevel(9);
    /// Service level 10.
    pub const SL10: ServiceLevel = ServiceLevel(10);
    /// Service level 11.
    pub const SL11: ServiceLevel = ServiceLevel(11);
    /// Service level 12.
    pub const SL12: ServiceLevel = ServiceLevel(12);
    /// Service level 13.
    pub const SL13: ServiceLevel = ServiceLevel(13);
    /// Service level 14.
    pub const SL14: ServiceLevel = ServiceLevel(14);
    /// Service level 15.
    pub const SL15: ServiceLevel = ServiceLevel(15);
}

impl Default for ServiceLevel {
    fn default() -> Self {
        ServiceLevel::SL0
    }
}

impl fmt::Display for ServiceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SL{}", self.0)
    }
}

impl TryFrom<u8> for ServiceLevel {
    type Error = ServiceLevelError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        ServiceLevel::new(value)
    }
}

impl From<ServiceLevel> for u8 {
    fn from(sl: ServiceLevel) -> Self {
        sl.as_u8()
    }
}

/// Error returned when an invalid service level value is provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceLevelError(pub u8);

impl fmt::Display for ServiceLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid service level {}: must be 0-15", self.0)
    }
}

impl std::error::Error for ServiceLevelError {}

// Serialize as the numeric value for API compatibility
impl Serialize for ServiceLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.0)
    }
}

// Deserialize from numeric value
impl<'de> Deserialize<'de> for ServiceLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        ServiceLevel::new(value).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_level_new() {
        assert!(ServiceLevel::new(0).is_ok());
        assert!(ServiceLevel::new(15).is_ok());
        assert!(ServiceLevel::new(16).is_err());
        assert!(ServiceLevel::new(255).is_err());
    }

    #[test]
    fn test_service_level_as_u8() {
        let sl = ServiceLevel::new(5).unwrap();
        assert_eq!(sl.as_u8(), 5);
    }

    #[test]
    fn test_service_level_constants() {
        assert_eq!(ServiceLevel::SL0.as_u8(), 0);
        assert_eq!(ServiceLevel::SL15.as_u8(), 15);
    }

    #[test]
    fn test_service_level_try_from() {
        assert_eq!(ServiceLevel::try_from(7u8), Ok(ServiceLevel::SL7));
        assert!(ServiceLevel::try_from(20u8).is_err());
    }

    #[test]
    fn test_service_level_serialization() {
        let sl = ServiceLevel::new(5).unwrap();
        let json = serde_json::to_string(&sl).unwrap();
        assert_eq!(json, "5");
    }

    #[test]
    fn test_service_level_deserialization() {
        let sl: ServiceLevel = serde_json::from_str("10").unwrap();
        assert_eq!(sl, ServiceLevel::SL10);
    }

    #[test]
    fn test_service_level_display() {
        assert_eq!(ServiceLevel::SL3.to_string(), "SL3");
    }
}
