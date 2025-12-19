//! GUID (Globally Unique Identifier) type for InfiniBand ports.
//!
//! GUIDs are 64-bit identifiers used to uniquely identify ports in an InfiniBand fabric.
//! They are typically represented as 16 hexadecimal characters.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;

/// A validated InfiniBand GUID.
///
/// GUIDs are 64-bit identifiers represented as 16 hexadecimal characters.
///
/// # Examples
///
/// ```
/// use ufm_client::types::Guid;
///
/// // Parse from string
/// let guid: Guid = "0002c903000e0b72".parse().unwrap();
///
/// // Or use the constructor
/// let guid = Guid::new("0002c903000e0b72").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Guid(String);

impl Guid {
    /// Create a new GUID from a string, validating the format.
    ///
    /// # Arguments
    ///
    /// * `value` - A string containing the GUID (16 hex characters)
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a valid GUID format.
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        Self::validate(&value)?;
        Ok(Self(value.to_lowercase()))
    }

    /// Create a GUID without validation (for internal use only)
    pub(crate) fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into().to_lowercase())
    }

    /// Validate that a string is a valid GUID format.
    fn validate(value: &str) -> Result<(), Error> {
        // GUIDs are 16 hex characters (64 bits)
        // Sometimes they may have colons or dashes as separators
        let cleaned: String = value.chars().filter(|c| c.is_ascii_hexdigit()).collect();

        if cleaned.len() != 16 {
            return Err(Error::InvalidGuid {
                value: value.to_string(),
            });
        }

        // Verify all characters are valid hex
        if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidGuid {
                value: value.to_string(),
            });
        }

        Ok(())
    }

    /// Get the raw string value of the GUID.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to the underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Guid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for Guid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // For deserialization from API, we accept the value as-is since UFM returned it
        Ok(Guid::new_unchecked(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_guid() {
        let guid = Guid::new("0002c903000e0b72").unwrap();
        assert_eq!(guid.as_str(), "0002c903000e0b72");
    }

    #[test]
    fn test_guid_case_insensitive() {
        let guid = Guid::new("0002C903000E0B72").unwrap();
        assert_eq!(guid.as_str(), "0002c903000e0b72");
    }

    #[test]
    fn test_invalid_guid_too_short() {
        assert!(Guid::new("0002c903").is_err());
    }

    #[test]
    fn test_invalid_guid_non_hex() {
        assert!(Guid::new("0002c903000e0bXX").is_err());
    }

    #[test]
    fn test_guid_from_str() {
        let guid: Guid = "0002c903000e0b72".parse().unwrap();
        assert_eq!(guid.as_str(), "0002c903000e0b72");
    }
}
