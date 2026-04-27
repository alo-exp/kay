//! Thin `reqwest::Client` wrapper for MiniMax's `/v1/text/chatcompletion_v2`
//! endpoint.
//!
//! MiniMax uses JSON-over-SSE streaming (not `text/event-stream` format).
//! Each line starts with `data: ` prefix containing a JSON object.

use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Url};

use crate::auth::ApiKey;
use crate::error::{AuthErrorKind, ProviderError};

pub(crate) struct MiniMaxClient {
    client: Client,
    endpoint: String,
    api_key: ApiKey,
}

impl std::fmt::Debug for MiniMaxClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiniMaxClient")
            .field("endpoint", &self.endpoint)
            .field("api_key", &self.api_key)
            .finish()
    }
}

impl MiniMaxClient {
    pub(crate) fn try_new(endpoint: String, api_key: ApiKey) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(ProviderError::Network)?;
        Ok(Self { client, endpoint, api_key })
    }

    /// Make a streaming POST request and return the raw bytes stream.
    pub(crate) async fn stream_chat(
        &self,
        body: Bytes,
    ) -> Result<reqwest::Response, ProviderError> {
        let headers = self.build_headers()?;
        let url = Url::parse(&self.endpoint).map_err(|_| ProviderError::Http {
            status: 0,
            body: format!("invalid endpoint URL: {}", self.endpoint),
        })?;
        let resp = self
            .client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(ProviderError::Network)?;

        // Check status before returning
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Http { status: status.as_u16(), body });
        }

        Ok(resp)
    }

    fn build_headers(&self) -> Result<HeaderMap, ProviderError> {
        let mut headers = HeaderMap::new();
        let auth = format!("Bearer {}", self.api_key.as_str());
        let auth_value = HeaderValue::from_str(&auth)
            .map_err(|_| ProviderError::Auth { reason: AuthErrorKind::Invalid })?;
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        Ok(headers)
    }
}
