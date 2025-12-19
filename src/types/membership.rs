//! Membership type for partition membership levels.
//!
//! Members of an InfiniBand partition can have either full or limited membership.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;

/// The membership level of a port in a partition.
///
/// In InfiniBand, partition members can have two levels of membership:
/// - **Full**: Can communicate with all other members (full and limited)
/// - **Limited**: Can only communicate with full members
///
/// # Examples
///
/// ```
/// use ufm_client::types::Membership;
///
/// let full = Membership::Full;
/// let limited = Membership::Limited;
///
/// // Parse from string
/// let membership: Membership = "full".parse().unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Membership {
    /// Full membership - can communicate with all members
    #[default]
    Full,
    /// Limited membership - can only communicate with full members
    Limited,
}

impl Membership {
    /// Returns true if this is full membership.
    pub fn is_full(&self) -> bool {
        matches!(self, Membership::Full)
    }

    /// Returns true if this is limited membership.
    pub fn is_limited(&self) -> bool {
        matches!(self, Membership::Limited)
    }

    /// Convert to the API string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Membership::Full => "full",
            Membership::Limited => "limited",
        }
    }
}

impl fmt::Display for Membership {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Membership {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" => Ok(Membership::Full),
            "limited" => Ok(Membership::Limited),
            _ => Err(Error::Configuration {
                message: format!("Invalid membership type: {s}"),
            }),
        }
    }
}

impl Serialize for Membership {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Membership {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        match s {
            Some(s) => s.parse().map_err(serde::de::Error::custom),
            None => Ok(Membership::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_membership_full() {
        let m = Membership::Full;
        assert!(m.is_full());
        assert!(!m.is_limited());
        assert_eq!(m.as_str(), "full");
    }

    #[test]
    fn test_membership_limited() {
        let m = Membership::Limited;
        assert!(m.is_limited());
        assert!(!m.is_full());
        assert_eq!(m.as_str(), "limited");
    }

    #[test]
    fn test_membership_from_str() {
        assert_eq!("full".parse::<Membership>().unwrap(), Membership::Full);
        assert_eq!("Full".parse::<Membership>().unwrap(), Membership::Full);
        assert_eq!(
            "limited".parse::<Membership>().unwrap(),
            Membership::Limited
        );
        assert_eq!(
            "LIMITED".parse::<Membership>().unwrap(),
            Membership::Limited
        );
    }

    #[test]
    fn test_membership_invalid() {
        assert!("invalid".parse::<Membership>().is_err());
    }
}
