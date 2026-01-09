//! The main UFM client implementation.
//!
//! This module provides the primary entry point for interacting with the UFM REST API.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use serde::de::DeserializeOwned;
use url::Url;

use crate::api::pkeys::PkeysApi;
use crate::api::version::VersionInfo;
use crate::api::vports::VportsApi;
use crate::auth::{Auth, SignRequest, TokenResponse};
use crate::error::{Error, Result};
use crate::http::{HttpClient, HttpConfig};

/// The base path for UFM REST API v1.
pub const UFM_REST_V1_PATH: &str = "/ufmRest";

/// The base path for UFM REST API v2.
pub const UFM_REST_V2_PATH: &str = "/ufmRestV2";

/// The main UFM client.
///
/// This client provides access to all UFM REST API operations.
///
/// # Thread Safety
///
/// `UfmClient` implements [`Clone`], [`Send`], and [`Sync`], making it safe to share
/// across async tasks. Cloning is cheap as the underlying HTTP client and
/// authentication are shared via [`Arc`].
///
/// # Examples
///
/// ```no_run
/// use ufm_client::{UfmClient, Auth};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = UfmClient::builder()
///     .base_url("https://ufm-server")
///     .auth(Auth::basic("admin", "password"))
///     .build()?;
///
/// // Get UFM version
/// let version = client.version().await?;
/// println!("UFM version: {}", version.ufm_release_version);
///
/// // Work with partition keys
/// let pkeys = client.pkeys().list().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct UfmClient {
    base_url: Url,
    http: Arc<HttpClient>,
    auth: Arc<dyn SignRequest>,
}

impl UfmClient {
    /// Create a new client builder.
    #[must_use]
    pub fn builder() -> UfmClientBuilder {
        UfmClientBuilder::default()
    }

    /// Create a client with basic authentication.
    ///
    /// This is a convenience method for creating a client with basic auth.
    /// For more configuration options, use [`UfmClient::builder()`].
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid or the HTTP client
    /// could not be created.
    #[must_use = "this returns a Result that should be checked"]
    pub fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self> {
        Self::builder()
            .base_url(base_url)
            .auth(Auth::basic(username, password))
            .build()
    }

    /// Get the base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Build a full URL for an API endpoint (V2).
    ///
    /// The path should start with `/` (e.g., `/app/ufm_version`).
    pub(crate) fn url(&self, path: &str) -> Result<Url> {
        // Ensure path starts with / for consistent URL joining
        debug_assert!(
            path.starts_with('/'),
            "API path should start with '/': {path}"
        );
        let full_path = format!("{UFM_REST_V2_PATH}{path}");
        self.base_url.join(&full_path).map_err(Error::from)
    }

    /// Build a full URL for a v1 API endpoint.
    ///
    /// The path should start with `/` (e.g., `/app/tokens`).
    pub(crate) fn url_v1(&self, path: &str) -> Result<Url> {
        debug_assert!(
            path.starts_with('/'),
            "API path should start with '/': {path}"
        );
        let full_path = format!("{UFM_REST_V1_PATH}{path}");
        self.base_url.join(&full_path).map_err(Error::from)
    }

    /// Execute a GET request.
    pub(crate) async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url(path)?;
        self.http
            .execute(Method::GET, url, self.auth.as_ref(), None)
            .await
    }

    /// Execute a POST request with a JSON body.
    pub(crate) async fn post<T>(&self, path: &str, body: serde_json::Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url(path)?;
        self.http
            .execute(Method::POST, url, self.auth.as_ref(), Some(body))
            .await
    }

    /// Execute a PUT request with a JSON body.
    pub(crate) async fn put<T>(&self, path: &str, body: serde_json::Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url(path)?;
        self.http
            .execute(Method::PUT, url, self.auth.as_ref(), Some(body))
            .await
    }

    /// Execute a DELETE request.
    pub(crate) async fn delete<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url(path)?;
        self.http
            .execute(Method::DELETE, url, self.auth.as_ref(), None)
            .await
    }

    /// Execute a DELETE request with a body.
    pub(crate) async fn delete_with_body<T>(&self, path: &str, body: serde_json::Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url(path)?;
        self.http
            .execute(Method::DELETE, url, self.auth.as_ref(), Some(body))
            .await
    }

    // ========== API Accessors ==========

    /// Access the partition keys (PKeys) API.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // List all PKeys
    /// let pkeys = client.pkeys().list().await?;
    ///
    /// // Get a specific PKey
    /// let pkey = "0x12".parse()?;
    /// let details = client.pkeys().get(&pkey).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn pkeys(&self) -> PkeysApi<'_> {
        PkeysApi::new(self)
    }

    /// Access the virtual ports (VPorts) API.
    ///
    /// VPorts are available when virtualization is enabled on the subnet manager.
    /// Use [`VportsApi::is_virtualization_enabled`] to check if virtualization is enabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Check if virtualization is enabled
    /// if client.vports().is_virtualization_enabled().await? {
    ///     // List all virtual ports
    ///     let vports = client.vports().list().await?;
    ///     for vport in vports {
    ///         println!("VPort: {}", vport.virtual_port_guid);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn vports(&self) -> VportsApi<'_> {
        VportsApi::new(self)
    }

    /// Get the UFM version information.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ufm_client::UfmClient;
    /// # async fn example(client: &UfmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let version = client.version().await?;
    /// println!("UFM version: {}", version.ufm_release_version);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn version(&self) -> Result<VersionInfo> {
        self.get("/app/ufm_version").await
    }

    /// Create an access token using basic authentication.
    ///
    /// This allows you to exchange username/password for a long-lived token.
    /// The returned token can be used with [`Auth::token()`] for subsequent requests.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails or the request cannot be completed.
    pub async fn create_token(&self) -> Result<TokenResponse> {
        let url = self.url_v1("/app/tokens")?;
        self.http
            .execute(Method::POST, url, self.auth.as_ref(), None)
            .await
    }
}

/// Builder for creating a [`UfmClient`].
///
/// # Required Configuration
///
/// - [`base_url`](Self::base_url) - The UFM server URL (required)
/// - [`auth`](Self::auth) - Authentication method (required)
///
/// # Optional Configuration
///
/// - [`timeout`](Self::timeout) - Request timeout (defaults to 30 seconds)
/// - [`max_retries`](Self::max_retries) - Retry count for transient failures (defaults to 3)
/// - [`verify_ssl`](Self::verify_ssl) - SSL certificate verification (defaults to `true`)
///
/// # Examples
///
/// ```no_run
/// use ufm_client::{UfmClient, Auth};
/// use std::time::Duration;
///
/// let client = UfmClient::builder()
///     .base_url("https://ufm-server")
///     .auth(Auth::basic("admin", "password"))
///     .timeout(Duration::from_secs(60))
///     .verify_ssl(false) // For self-signed certs
///     .build()
///     .expect("Failed to create client");
/// ```
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct UfmClientBuilder {
    base_url: Option<String>,
    auth: Option<Auth>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    verify_ssl: Option<bool>,
}

impl UfmClientBuilder {
    /// Set the base URL for the UFM server.
    ///
    /// This should be the root URL without any path, e.g., `https://ufm-server:443`.
    /// The URL will be normalized to ensure proper path joining.
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the authentication method.
    ///
    /// This is required. See [`Auth`] for available authentication methods:
    /// - [`Auth::basic`] - Username and password authentication
    /// - [`Auth::token`] - Token-based authentication
    #[must_use]
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set the request timeout.
    ///
    /// Defaults to 30 seconds if not specified.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of retries for failed requests.
    ///
    /// Only retryable errors (network errors, timeouts, rate limits) will be retried.
    /// See [`Error::is_retryable`](crate::Error::is_retryable) for details.
    ///
    /// Defaults to 3 retries if not specified.
    #[must_use]
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Set whether to verify SSL certificates.
    ///
    /// Set to `false` to allow self-signed certificates.
    ///
    /// # Security Warning
    ///
    /// Disabling SSL verification is insecure and should only be used for
    /// development/testing environments.
    #[must_use]
    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = Some(verify);
        self
    }

    /// Build the client.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No base URL was provided
    /// - No authentication was provided
    /// - The base URL is invalid
    /// - The HTTP client could not be created
    pub fn build(self) -> Result<UfmClient> {
        let base_url_str = self
            .base_url
            .ok_or_else(|| Error::configuration("base_url is required"))?;

        let auth_config = self
            .auth
            .ok_or_else(|| Error::configuration("auth is required"))?;

        // Parse and normalize the base URL to ensure consistent path joining
        let mut base_url = Url::parse(&base_url_str)
            .map_err(|e| Error::configuration(format!("Invalid base URL '{base_url_str}': {e}")))?;

        // Ensure the base URL has a trailing slash for proper path joining
        // This prevents issues with Url::join when the base has a path component
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }

        let http_config = HttpConfig {
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            max_retries: self.max_retries.unwrap_or(3),
            verify_ssl: self.verify_ssl.unwrap_or(true),
            ..Default::default()
        };

        let http = Arc::new(HttpClient::new(http_config)?);
        let auth = auth_config.into_signer();

        Ok(UfmClient {
            base_url,
            http,
            auth,
        })
    }
}
