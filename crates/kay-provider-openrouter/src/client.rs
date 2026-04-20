//! Thin `reqwest::Client` wrapper for OpenRouter's `/api/v1/chat/completions`
//! endpoint (PROV-02).
//!
//! Responsibilities:
//!   - Own the API key + optional identity headers (HTTP-Referer, X-Title).
//!   - Build a single POST + SSE `EventSource`; surface `ProviderError` for
//!     config-level failures before the stream opens.
//!   - Disable `reqwest_eventsource`'s built-in exponential retry so
//!     `backon` (plan 02-10) is the single source of retry truth (Pitfall 6).
//!
//! NO retry orchestration here — plan 02-10 wraps this behind backon.
//! NO `.unwrap()` / `.expect()` — crate-wide lint forbids them.

use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Url};
use reqwest_eventsource::{EventSource, RequestBuilderExt, retry};

use crate::auth::ApiKey;
use crate::error::{AuthErrorKind, ProviderError};

pub(crate) struct UpstreamClient {
    client: Client,
    endpoint: Url,
    api_key: ApiKey,
    referer: Option<String>,
    title: Option<String>,
}

impl UpstreamClient {
    /// Fallible constructor. Returns `ProviderError::Network` if reqwest
    /// cannot build its internal client (e.g. TLS root-store init failure).
    /// No infallible `new()` variant exists by design — crate lint forbids
    /// `.unwrap()` / `.expect()` at the call site.
    pub(crate) fn try_new(endpoint: Url, api_key: ApiKey) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(ProviderError::Network)?;
        Ok(Self { client, endpoint, api_key, referer: None, title: None })
    }

    /// Attach optional OpenRouter identity headers (recommended by OpenRouter
    /// docs and the upstream ForgeCode provider). Both are optional — unset
    /// values are simply not sent.
    pub(crate) fn with_headers(mut self, referer: Option<String>, title: Option<String>) -> Self {
        self.referer = referer;
        self.title = title;
        self
    }

    /// POST a JSON body and return the SSE `EventSource`. Non-2xx responses
    /// surface inside the event stream as `Error::InvalidStatusCode` — the
    /// translator (or plan 02-10's classifier) maps those to `ProviderError`.
    /// Transport failures that happen BEFORE the stream opens (DNS, TCP,
    /// TLS handshake) surface as `ProviderError::Network`.
    pub(crate) async fn stream_chat(&self, body: Bytes) -> Result<EventSource, ProviderError> {
        let headers = self.build_headers()?;
        let req = self
            .client
            .post(self.endpoint.clone())
            .headers(headers)
            .body(body);

        // WARNING #4 / Pitfall 6: disable EventSource's built-in exponential
        // retry so backon (plan 02-10) is the single retry orchestrator.
        let mut es = req
            .eventsource()
            .map_err(|e| ProviderError::Stream(format!("eventsource setup: {e}")))?;
        es.set_retry_policy(Box::new(retry::Never));
        Ok(es)
    }

    /// Build the HeaderMap for every request. Authorization is mandatory;
    /// the two identity headers are optional. Any `HeaderValue::from_str`
    /// failure on the bearer token maps to `AuthErrorKind::Invalid` — the
    /// key contains bytes that cannot be sent as an HTTP header (e.g. a
    /// stray newline — which charset validation in Allowlist also catches).
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
        if let Some(ref r) = self.referer
            && let Ok(v) = HeaderValue::from_str(r)
        {
            headers.insert(HeaderName::from_static("http-referer"), v);
        }
        if let Some(ref t) = self.title
            && let Ok(v) = HeaderValue::from_str(t)
        {
            headers.insert(HeaderName::from_static("x-title"), v);
        }
        Ok(headers)
    }

    #[cfg(test)]
    #[allow(dead_code)] // test-only diagnostic accessor; retained for future cases
    pub(crate) fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod unit {
    use super::*;

    fn test_key() -> ApiKey {
        ApiKey::from("sk-test".to_string())
    }

    #[test]
    fn try_new_succeeds_for_valid_url() {
        let url = Url::parse("https://openrouter.ai/api/v1/chat/completions").unwrap();
        assert!(UpstreamClient::try_new(url, test_key()).is_ok());
    }

    #[test]
    fn headers_contain_authorization_and_content_type() {
        let url = Url::parse("https://openrouter.ai/api/v1/chat/completions").unwrap();
        let c = UpstreamClient::try_new(url, test_key()).unwrap();
        let h = c.build_headers().unwrap();
        assert!(h.contains_key(reqwest::header::AUTHORIZATION));
        assert!(h.contains_key(reqwest::header::CONTENT_TYPE));
        // Credential IS the header value — that is transport-necessary.
        // Consumer code MUST NOT log this HeaderMap (TM-01).
    }

    #[test]
    fn with_headers_sets_optional_identity_headers() {
        let url = Url::parse("https://openrouter.ai/api/v1/chat/completions").unwrap();
        let c = UpstreamClient::try_new(url, test_key())
            .unwrap()
            .with_headers(Some("https://kay.dev".into()), Some("Kay".into()));
        let h = c.build_headers().unwrap();
        assert!(h.contains_key(HeaderName::from_static("http-referer")));
        assert!(h.contains_key(HeaderName::from_static("x-title")));
    }
}
