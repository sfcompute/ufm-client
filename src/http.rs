//! HTTP request/response handling for the UFM client.
//!
//! This module provides low-level HTTP operations, request building,
//! and response parsing.

use std::time::Duration;

use reqwest::{Certificate, Client as ReqwestClient, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use tracing::{debug, error, trace, warn};
use url::Url;

use crate::auth::SignRequest;
use crate::error::{ApiErrorResponse, Error, Result};

/// Default request timeout in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default number of retries for retryable errors.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Base delay between retries in milliseconds.
pub const DEFAULT_RETRY_DELAY_MS: u64 = 1000;

/// HTTP configuration for the client.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of retries for retryable errors
    pub max_retries: u32,
    /// Base delay between retries
    pub retry_delay: Duration,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
    /// PEM bundle of the only roots to trust; `None` uses the default roots
    pub root_certificates_pem: Option<Vec<u8>>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
            retry_delay: Duration::from_millis(DEFAULT_RETRY_DELAY_MS),
            verify_ssl: true,
            root_certificates_pem: None,
        }
    }
}

/// Internal HTTP client wrapper.
#[derive(Debug)]
pub(crate) struct HttpClient {
    client: ReqwestClient,
    config: HttpConfig,
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration.
    pub fn new(config: HttpConfig) -> Result<Self> {
        let mut builder = ReqwestClient::builder()
            .use_rustls_tls()
            .timeout(config.timeout)
            .danger_accept_invalid_certs(!config.verify_ssl)
            .pool_max_idle_per_host(10);
        if let Some(pem) = &config.root_certificates_pem {
            let roots = Certificate::from_pem_bundle(pem)
                .map_err(|e| Error::configuration(format!("Invalid root certificates PEM: {e}")))?;
            if roots.is_empty() {
                return Err(Error::configuration(
                    "root certificates PEM contains no certificates",
                ));
            }
            builder = roots
                .into_iter()
                .fold(builder.tls_built_in_root_certs(false), |b, root| {
                    b.add_root_certificate(root)
                });
        }
        let client = builder.build().map_err(Error::Http)?;

        Ok(Self { client, config })
    }

    /// Execute a request with retry logic.
    pub async fn execute<T: DeserializeOwned>(
        &self,
        method: Method,
        url: Url,
        auth: &dyn SignRequest,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        let mut last_error = None;
        let mut retry_count = 0;

        while retry_count <= self.config.max_retries {
            if retry_count > 0 {
                let delay = self.config.retry_delay * retry_count;
                debug!(
                    "Retrying request after {:?} (attempt {})",
                    delay, retry_count
                );
                tokio::time::sleep(delay).await;
            }

            match self
                .execute_once(method.clone(), url.clone(), auth, body.clone())
                .await
            {
                Ok(response) => return self.parse_response(response).await,
                Err(e) if e.is_retryable() && retry_count < self.config.max_retries => {
                    warn!("Request failed with retryable error: {}, retrying...", e);
                    last_error = Some(e);
                    retry_count += 1;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| Error::configuration("Max retries exceeded")))
    }

    async fn execute_once(
        &self,
        method: Method,
        url: Url,
        auth: &dyn SignRequest,
        body: Option<serde_json::Value>,
    ) -> Result<Response> {
        debug!("Executing {} {}", method, url);
        if let Some(ref json_body) = body {
            trace!("Request body: {}", json_body);
        }

        let mut request = self.client.request(method.clone(), url.clone());
        request = auth.sign(request);

        if let Some(json_body) = body {
            request = request.json(&json_body);
        }

        request.send().await.map_err(|e| {
            error!("Request failed for {} {}: {}", method, url, e);
            if e.is_timeout() {
                Error::Timeout
            } else {
                Error::Http(e)
            }
        })
    }

    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        let url = response.url().to_string();
        debug!("Response status: {} for {}", status, url);

        if status.is_success() {
            let text = response.text().await.map_err(Error::Http)?;
            trace!("Response body: {}", text);

            // Handle empty response bodies by treating them as null
            // Many UFM API endpoints return empty responses on success
            let json_text = if text.trim().is_empty() {
                "null"
            } else {
                &text
            };

            serde_json::from_str(json_text).map_err(|e| {
                // Log the raw response when JSON parsing fails - very helpful for debugging
                error!(
                    "JSON parsing error for {}: {}. Response body (first 500 chars): '{}'",
                    url,
                    e,
                    &text[..text.len().min(500)]
                );
                Error::Json(e)
            })
        } else {
            self.handle_error_response(response, status).await
        }
    }

    async fn handle_error_response<T>(&self, response: Response, status: StatusCode) -> Result<T> {
        let status_code = status.as_u16();

        // Try to parse error response body
        let error_message = match response.text().await {
            Ok(text) => {
                if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&text) {
                    error_response.to_string()
                } else if !text.is_empty() {
                    text
                } else {
                    status
                        .canonical_reason()
                        .unwrap_or("Unknown error")
                        .to_string()
                }
            }
            Err(_) => status
                .canonical_reason()
                .unwrap_or("Unknown error")
                .to_string(),
        };

        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(Error::authentication(error_message))
            }
            StatusCode::NOT_FOUND => Err(Error::not_found(error_message)),
            StatusCode::TOO_MANY_REQUESTS => Err(Error::RateLimited {
                retry_after_secs: 60,
            }),
            _ => Err(Error::api(status_code, error_message)),
        }
    }
}
