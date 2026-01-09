//! Authentication for the UFM REST API.
//!
//! UFM supports multiple authentication methods:
//! - Basic authentication (username:password)
//! - Token-based authentication (access tokens)
//! - Session cookies (cookie-based auth)
//!
//! This module provides a pluggable authentication system that can be
//! configured at client creation time.

use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::RequestBuilder;
use secrecy::{ExposeSecret, SecretString};

/// Trait for types that can sign HTTP requests with authentication.
///
/// Implement this trait to add custom authentication methods.
pub trait SignRequest: Send + Sync + std::fmt::Debug {
    /// Apply authentication to the request builder.
    fn sign(&self, request: RequestBuilder) -> RequestBuilder;

    /// Get the authentication headers (for debugging/logging).
    fn auth_headers(&self) -> HeaderMap;
}

/// Authentication configuration for the UFM client.
///
/// UFM requires authentication for all API operations. Choose either
/// basic authentication (username/password) or token authentication.
///
/// # Examples
///
/// ```
/// use ufm_client::auth::Auth;
///
/// // Basic auth
/// let auth = Auth::basic("admin", "password");
///
/// // Token auth
/// let auth = Auth::token("your-access-token");
/// ```
#[derive(Debug, Clone)]
pub enum Auth {
    /// Basic HTTP authentication with username and password
    Basic {
        /// The username
        username: String,
        /// The password (stored securely)
        password: SecretString,
    },
    /// Token-based authentication
    Token(SecretString),
}

impl Auth {
    /// Create basic authentication credentials.
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Auth::Basic {
            username: username.into(),
            password: SecretString::from(password.into()),
        }
    }

    /// Create token-based authentication.
    pub fn token(token: impl Into<String>) -> Self {
        Auth::Token(SecretString::from(token.into()))
    }

    /// Convert to an Arc<dyn SignRequest> for use in the client.
    pub fn into_signer(self) -> Arc<dyn SignRequest> {
        Arc::new(self)
    }
}

impl SignRequest for Auth {
    fn sign(&self, request: RequestBuilder) -> RequestBuilder {
        match self {
            Auth::Basic { username, password } => {
                request.basic_auth(username, Some(password.expose_secret()))
            }
            Auth::Token(token) => {
                // UFM uses "Authorization: Basic <token>" for token auth
                // (not Bearer, despite being a token)
                let auth_value = format!("Basic {}", token.expose_secret());
                request.header(AUTHORIZATION, auth_value)
            }
        }
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        match self {
            Auth::Basic { username, .. } => {
                // Don't expose the actual auth header, just indicate type
                headers.insert("X-Auth-Type", HeaderValue::from_static("basic"));
                if let Ok(v) = HeaderValue::from_str(username) {
                    headers.insert("X-Auth-User", v);
                }
            }
            Auth::Token(_) => {
                headers.insert("X-Auth-Type", HeaderValue::from_static("token"));
            }
        }
        headers
    }
}

/// Token response from the UFM token creation endpoint.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TokenResponse {
    /// The access token
    pub access_token: String,
    /// Whether the token has been revoked
    pub revoked: bool,
    /// Unix timestamp when the token was issued
    pub issued_at: i64,
    /// Token lifetime in seconds
    pub expires_in: i64,
    /// The username associated with the token
    pub username: String,
}

impl TokenResponse {
    /// Check if the token is valid (not revoked).
    pub fn is_valid(&self) -> bool {
        !self.revoked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth() {
        let auth = Auth::basic("admin", "password123");
        match auth {
            Auth::Basic { username, .. } => {
                assert_eq!(username, "admin");
            }
            _ => panic!("Expected Basic auth"),
        }
    }

    #[test]
    fn test_token_auth() {
        let auth = Auth::token("my-token");
        assert!(matches!(auth, Auth::Token(_)));
    }

    #[test]
    fn test_auth_headers_basic() {
        let auth = Auth::basic("admin", "pass");
        let headers = auth.auth_headers();
        assert_eq!(headers.get("X-Auth-Type").unwrap(), "basic");
        assert_eq!(headers.get("X-Auth-User").unwrap(), "admin");
    }
}
