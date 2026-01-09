//! MTU (Maximum Transmission Unit) limit types for QoS configuration.
//!
//! This module provides a strongly-typed enum for MTU limits used in partition
//! QoS configuration. The MTU limit controls the maximum packet size for
//! partition traffic.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// MTU (Maximum Transmission Unit) limit for partition traffic.
///
/// The MTU limit controls the maximum packet size for traffic on a partition.
/// Each variant corresponds to a specific byte size.
///
/// # Examples
///
/// ```
/// use ufm_client::types::MtuLimit;
///
/// let mtu = MtuLimit::Mtu2048;
/// assert_eq!(mtu.as_u8(), 4);
/// assert_eq!(mtu.bytes(), 2048);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MtuLimit {
    /// 256 bytes MTU (value: 1)
    Mtu256 = 1,
    /// 512 bytes MTU (value: 2)
    Mtu512 = 2,
    /// 1024 bytes MTU (value: 3)
    Mtu1024 = 3,
    /// 2048 bytes MTU (value: 4)
    Mtu2048 = 4,
    /// 4096 bytes MTU (value: 5)
    Mtu4096 = 5,
}

impl MtuLimit {
    /// Convert the MTU limit to its API representation (1-5).
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get the actual MTU size in bytes.
    #[inline]
    pub const fn bytes(self) -> u16 {
        match self {
            MtuLimit::Mtu256 => 256,
            MtuLimit::Mtu512 => 512,
            MtuLimit::Mtu1024 => 1024,
            MtuLimit::Mtu2048 => 2048,
            MtuLimit::Mtu4096 => 4096,
        }
    }

    /// Create an MTU limit from a u8 value (1-5).
    ///
    /// Returns `None` if the value is outside the valid range.
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(MtuLimit::Mtu256),
            2 => Some(MtuLimit::Mtu512),
            3 => Some(MtuLimit::Mtu1024),
            4 => Some(MtuLimit::Mtu2048),
            5 => Some(MtuLimit::Mtu4096),
            _ => None,
        }
    }
}

impl fmt::Display for MtuLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bytes", self.bytes())
    }
}

impl TryFrom<u8> for MtuLimit {
    type Error = MtuLimitError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        MtuLimit::from_u8(value).ok_or(MtuLimitError(value))
    }
}

impl From<MtuLimit> for u8 {
    fn from(mtu: MtuLimit) -> Self {
        mtu.as_u8()
    }
}

/// Error returned when an invalid MTU limit value is provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MtuLimitError(pub u8);

impl fmt::Display for MtuLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid MTU limit value {}: must be 1-5 (256, 512, 1024, 2048, or 4096 bytes)",
            self.0
        )
    }
}

impl std::error::Error for MtuLimitError {}

// Serialize as the numeric value for API compatibility
impl Serialize for MtuLimit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}

// Deserialize from numeric value
impl<'de> Deserialize<'de> for MtuLimit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        MtuLimit::from_u8(value).ok_or_else(|| {
            serde::de::Error::custom(format!("invalid MTU limit value {value}: must be 1-5"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtu_as_u8() {
        assert_eq!(MtuLimit::Mtu256.as_u8(), 1);
        assert_eq!(MtuLimit::Mtu512.as_u8(), 2);
        assert_eq!(MtuLimit::Mtu1024.as_u8(), 3);
        assert_eq!(MtuLimit::Mtu2048.as_u8(), 4);
        assert_eq!(MtuLimit::Mtu4096.as_u8(), 5);
    }

    #[test]
    fn test_mtu_bytes() {
        assert_eq!(MtuLimit::Mtu256.bytes(), 256);
        assert_eq!(MtuLimit::Mtu512.bytes(), 512);
        assert_eq!(MtuLimit::Mtu1024.bytes(), 1024);
        assert_eq!(MtuLimit::Mtu2048.bytes(), 2048);
        assert_eq!(MtuLimit::Mtu4096.bytes(), 4096);
    }

    #[test]
    fn test_mtu_from_u8() {
        assert_eq!(MtuLimit::from_u8(1), Some(MtuLimit::Mtu256));
        assert_eq!(MtuLimit::from_u8(4), Some(MtuLimit::Mtu2048));
        assert_eq!(MtuLimit::from_u8(0), None);
        assert_eq!(MtuLimit::from_u8(6), None);
    }

    #[test]
    fn test_mtu_try_from() {
        assert_eq!(MtuLimit::try_from(3u8), Ok(MtuLimit::Mtu1024));
        assert!(MtuLimit::try_from(10u8).is_err());
    }

    #[test]
    fn test_mtu_serialization() {
        let mtu = MtuLimit::Mtu2048;
        let json = serde_json::to_string(&mtu).unwrap();
        assert_eq!(json, "4");
    }

    #[test]
    fn test_mtu_deserialization() {
        let mtu: MtuLimit = serde_json::from_str("3").unwrap();
        assert_eq!(mtu, MtuLimit::Mtu1024);
    }

    #[test]
    fn test_mtu_display() {
        assert_eq!(MtuLimit::Mtu2048.to_string(), "2048 bytes");
    }
}
