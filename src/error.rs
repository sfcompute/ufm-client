//! Error types for the UFM client library.
//!
//! This module provides strongly-typed errors for all failure modes
//! that can occur when interacting with the UFM REST API.

use std::fmt;
use thiserror::Error;

/// The main error type for UFM client operations.
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request/response error from reqwest
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication error
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    /// Invalid GUID format
    #[error("Invalid GUID format: {value}")]
    InvalidGuid { value: String },

    /// Invalid PKey format
    #[error("Invalid PKey format: {value}")]
    InvalidPkey { value: String },

    /// API returned an error response
    #[error("API error (HTTP {status_code}): {message}")]
    Api { status_code: u16, message: String },

    /// Resource not found
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    /// Client configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    /// Request timeout
    #[error("Request timed out")]
    Timeout,

    /// Virtualization is not enabled on the UFM subnet manager
    #[error("Virtualization is not enabled on the UFM subnet manager")]
    VirtualizationNotEnabled,

    /// Feature not supported by this UFM version
    #[error("Feature not supported: {feature}")]
    FeatureNotSupported { feature: String },
}

impl Error {
    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create an API error from HTTP status code and message
    pub fn api(status_code: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status_code,
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a feature not supported error
    pub fn feature_not_supported(feature: impl Into<String>) -> Self {
        Self::FeatureNotSupported {
            feature: feature.into(),
        }
    }

    /// Check if this is a virtualization not enabled error
    pub fn is_virtualization_error(&self) -> bool {
        matches!(self, Error::VirtualizationNotEnabled)
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Http(_) | Error::Timeout | Error::RateLimited { .. }
        )
    }

    /// Check if this is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Authentication { .. })
            || matches!(self, Error::Api { status_code, .. } if *status_code == 401 || *status_code == 403)
    }

    /// Check if this is a not found error (404)
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound { .. })
            || matches!(self, Error::Api { status_code, .. } if *status_code == 404)
    }
}

/// A specialized Result type for UFM client operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error response from the UFM API
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiErrorResponse {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

impl fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref error) = self.error {
            write!(f, "{error}")
        } else if let Some(ref message) = self.message {
            write!(f, "{message}")
        } else if let Some(ref detail) = self.detail {
            write!(f, "{detail}")
        } else {
            write!(f, "Unknown error")
        }
    }
}
