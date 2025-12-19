//! UFM version API.
//!
//! This module provides access to UFM version information.

use serde::Deserialize;
use std::collections::HashMap;

/// UFM version information.
///
/// Contains version information for various UFM components.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VersionInfo {
    /// OpenSM version
    #[serde(default)]
    pub opensm: Option<String>,

    /// SHARP version
    #[serde(default)]
    pub sharp: Option<String>,

    /// IBDiagnet version
    #[serde(default)]
    pub ibdiagnet: Option<String>,

    /// Telemetry version
    #[serde(default)]
    pub telemetry: Option<String>,

    /// MFT version
    #[serde(default)]
    pub mft: Option<String>,

    /// UFM Web UI version
    #[serde(default)]
    pub ufm_webui: Option<String>,

    /// UFM release version (e.g., "6.18.0-5")
    #[serde(default)]
    pub ufm_release_version: String,

    /// Plugin versions
    #[serde(default)]
    pub plugins: HashMap<String, String>,
}

impl VersionInfo {
    /// Parse the major version number from the release version.
    pub fn major_version(&self) -> Option<u32> {
        self.ufm_release_version
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
    }

    /// Parse the minor version number from the release version.
    pub fn minor_version(&self) -> Option<u32> {
        self.ufm_release_version
            .split('.')
            .nth(1)
            .and_then(|s| s.parse().ok())
    }

    /// Check if the UFM version is at least the specified version.
    pub fn is_at_least(&self, major: u32, minor: u32) -> bool {
        match (self.major_version(), self.minor_version()) {
            (Some(maj), Some(min)) => maj > major || (maj == major && min >= minor),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let json = r#"{
            "opensm": "5.20.0_ef1f438",
            "sharp": "3.8.0-rc5",
            "ufm_release_version": "6.18.0-5",
            "plugins": {}
        }"#;

        let info: VersionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.ufm_release_version, "6.18.0-5");
        assert_eq!(info.major_version(), Some(6));
        assert_eq!(info.minor_version(), Some(18));
    }

    #[test]
    fn test_is_at_least() {
        let info = VersionInfo {
            opensm: None,
            sharp: None,
            ibdiagnet: None,
            telemetry: None,
            mft: None,
            ufm_webui: None,
            ufm_release_version: "6.18.0-5".to_string(),
            plugins: HashMap::new(),
        };

        assert!(info.is_at_least(6, 17));
        assert!(info.is_at_least(6, 18));
        assert!(!info.is_at_least(6, 19));
        assert!(!info.is_at_least(7, 0));
    }
}
