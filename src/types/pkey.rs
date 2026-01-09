//! Partition Key (PKey) type for InfiniBand partitions.
//!
//! PKeys are 16-bit values that identify partitions in an InfiniBand fabric.
//! They are typically represented as hexadecimal strings prefixed with "0x".

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;

/// A validated InfiniBand Partition Key (PKey).
///
/// PKeys are 16-bit values (0x0001 to 0xFFFF) that identify partitions.
/// The value 0x7FFF is typically reserved as the default management PKey.
///
/// # Examples
///
/// ```
/// use ufm_client::types::Pkey;
///
/// // Parse from hex string
/// let pkey: Pkey = "0x12".parse().unwrap();
///
/// // Or use the constructor
/// let pkey = Pkey::new("0x0a12").unwrap();
///
/// // Create from u16 value
/// let pkey = Pkey::from_value(0x12);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pkey(u16);

impl Pkey {
    /// The default management PKey value.
    pub const DEFAULT_MANAGEMENT: u16 = 0x7FFF;

    /// Create a new PKey from a hex string.
    ///
    /// # Arguments
    ///
    /// * `value` - A hex string like "0x12" or "0x0a12"
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a valid PKey format.
    pub fn new(value: impl AsRef<str>) -> Result<Self, Error> {
        let value = value.as_ref();
        let parsed = Self::parse_hex(value)?;
        Self::validate(parsed)?;
        Ok(Self(parsed))
    }

    /// Create a PKey from a raw u16 value.
    ///
    /// # Panics
    ///
    /// Panics if the value is 0 (reserved).
    pub fn from_value(value: u16) -> Self {
        Self::try_from_value(value).expect("PKey value cannot be 0")
    }

    /// Try to create a PKey from a raw u16 value.
    pub fn try_from_value(value: u16) -> Result<Self, Error> {
        Self::validate(value)?;
        Ok(Self(value))
    }

    /// Create the default management PKey.
    pub fn default_management() -> Self {
        Self(Self::DEFAULT_MANAGEMENT)
    }

    fn parse_hex(value: &str) -> Result<u16, Error> {
        let cleaned = value.trim().to_lowercase();

        // Handle "0x" prefix
        let hex_str = cleaned.strip_prefix("0x").unwrap_or(&cleaned);

        u16::from_str_radix(hex_str, 16).map_err(|_| Error::InvalidPkey {
            value: value.to_string(),
        })
    }

    fn validate(value: u16) -> Result<(), Error> {
        // PKey 0 is reserved/invalid
        if value == 0 {
            return Err(Error::InvalidPkey {
                value: "0".to_string(),
            });
        }
        Ok(())
    }

    /// Get the raw u16 value.
    pub fn value(&self) -> u16 {
        self.0
    }

    /// Format as a hex string with "0x" prefix.
    pub fn as_hex_string(&self) -> String {
        format!("0x{:x}", self.0)
    }

    /// Check if this is the default management PKey.
    pub fn is_management_pkey(&self) -> bool {
        self.0 == Self::DEFAULT_MANAGEMENT
    }
}

impl fmt::Display for Pkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl FromStr for Pkey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Serialize for Pkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_hex_string())
    }
}

impl<'de> Deserialize<'de> for Pkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Pkey::new(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pkey() {
        let pkey = Pkey::new("0x12").unwrap();
        assert_eq!(pkey.value(), 0x12);
        assert_eq!(pkey.as_hex_string(), "0x12");
    }

    #[test]
    fn test_pkey_with_leading_zeros() {
        let pkey = Pkey::new("0x0a12").unwrap();
        assert_eq!(pkey.value(), 0x0a12);
    }

    #[test]
    fn test_pkey_case_insensitive() {
        let pkey1 = Pkey::new("0xABCD").unwrap();
        let pkey2 = Pkey::new("0xabcd").unwrap();
        assert_eq!(pkey1, pkey2);
    }

    #[test]
    fn test_invalid_pkey_zero() {
        assert!(Pkey::new("0x0").is_err());
    }

    #[test]
    fn test_pkey_from_value() {
        let pkey = Pkey::from_value(0x12);
        assert_eq!(pkey.value(), 0x12);
    }

    #[test]
    fn test_management_pkey() {
        let pkey = Pkey::default_management();
        assert!(pkey.is_management_pkey());
        assert_eq!(pkey.value(), 0x7FFF);
    }
}
