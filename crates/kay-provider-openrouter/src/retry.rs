//! Retry policy + Retry-After parsing + HTTP-status classification.
//! (PROV-07, PROV-08, D-09).
//!
//! backon::ExponentialBuilder defaults per D-09: base 500ms, factor 2,
//! max 3 attempts, max delay 8s, full jitter.
//!
//! Retry-After (integer seconds only; HTTP-date form is spec-allowed
//! but rare — RESEARCH §A4 flags this as a deferred edge case) takes
//! precedence over the backon calculation when the upstream response
//! is 429.

use std::time::Duration;

use backon::ExponentialBuilder;
use reqwest::header::HeaderMap;

use crate::error::{AuthErrorKind, ProviderError};

/// D-09 default retry schedule: 500ms base, 2x factor, 8s cap, 3 attempts,
/// full jitter.
pub(crate) fn default_backoff() -> ExponentialBuilder {
    ExponentialBuilder::default()
        .with_min_delay(Duration::from_millis(500))
        .with_factor(2.0)
        .with_max_delay(Duration::from_secs(8))
        .with_max_times(3)
        .with_jitter()
}

/// Parse the Retry-After header as integer seconds.
///
/// Returns `None` for absent, unparseable, or HTTP-date form (the latter
/// is spec-allowed per RFC 7231 §7.1.3 but rare on OpenRouter per RESEARCH
/// §A4 / §Pitfall 6). When `None`, callers fall back to backon's default
/// schedule.
pub(crate) fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
}

/// Classify an HTTP response into a typed `ProviderError` (PROV-08).
///
/// Mapping:
///   - 401           → `Auth { reason: AuthErrorKind::Invalid }`
///   - 429           → `RateLimited { retry_after: parse_retry_after(...) }`
///   - 500..=599     → `ServerError { status }`
///   - otherwise     → `Http { status, body }`
///
/// Body is preserved verbatim for `Http` surfaces; callers MUST NOT log
/// it unbounded (see CONTEXT.md threat model).
pub(crate) fn classify_http_error(
    status: u16,
    headers: &HeaderMap,
    body: String,
) -> ProviderError {
    match status {
        401 => ProviderError::Auth {
            reason: AuthErrorKind::Invalid,
        },
        429 => ProviderError::RateLimited {
            retry_after: parse_retry_after(headers),
        },
        500..=599 => ProviderError::ServerError { status },
        _ => ProviderError::Http { status, body },
    }
}

/// Retry predicate. `RateLimited` / `ServerError` / `Network` are the
/// three retryable classes; everything else is terminal.
pub(crate) fn is_retryable(err: &ProviderError) -> bool {
    matches!(
        err,
        ProviderError::RateLimited { .. }
            | ProviderError::ServerError { .. }
            | ProviderError::Network(_)
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod unit {
    use super::*;

    use reqwest::header::{HeaderValue, RETRY_AFTER};

    #[test]
    fn default_backoff_builds_without_panic() {
        // backon does not expose attempt-count getters; exercise the
        // builder to confirm construction is infallible. Integration
        // tests assert the 3-attempt behavior via mockito hit counts.
        let _ = default_backoff();
    }

    #[test]
    fn parse_retry_after_integer_seconds() {
        let mut h = HeaderMap::new();
        h.insert(RETRY_AFTER, HeaderValue::from_static("5"));
        assert_eq!(parse_retry_after(&h), Some(Duration::from_secs(5)));
    }

    #[test]
    fn parse_retry_after_missing_returns_none() {
        let h = HeaderMap::new();
        assert_eq!(parse_retry_after(&h), None);
    }

    #[test]
    fn parse_retry_after_date_format_returns_none() {
        // HTTP-date form is spec-allowed but rare; we return None so
        // backon's default schedule applies (RESEARCH §A4).
        let mut h = HeaderMap::new();
        h.insert(
            RETRY_AFTER,
            HeaderValue::from_static("Wed, 21 Oct 2026 07:28:00 GMT"),
        );
        assert_eq!(parse_retry_after(&h), None);
    }

    #[test]
    fn classify_401_is_auth_invalid() {
        let e = classify_http_error(401, &HeaderMap::new(), "bad key".into());
        assert!(matches!(
            e,
            ProviderError::Auth {
                reason: AuthErrorKind::Invalid
            }
        ));
    }

    #[test]
    fn classify_402_is_http_body_preserved() {
        let e = classify_http_error(402, &HeaderMap::new(), "insufficient credits".into());
        match e {
            ProviderError::Http { status, body } => {
                assert_eq!(status, 402);
                assert_eq!(body, "insufficient credits");
            }
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[test]
    fn classify_429_uses_retry_after() {
        let mut h = HeaderMap::new();
        h.insert(RETRY_AFTER, HeaderValue::from_static("10"));
        let e = classify_http_error(429, &h, String::new());
        match e {
            ProviderError::RateLimited { retry_after } => {
                assert_eq!(retry_after, Some(Duration::from_secs(10)));
            }
            other => panic!("expected RateLimited, got {other:?}"),
        }
    }

    #[test]
    fn classify_429_without_retry_after_is_none() {
        let e = classify_http_error(429, &HeaderMap::new(), String::new());
        match e {
            ProviderError::RateLimited { retry_after } => {
                assert_eq!(retry_after, None);
            }
            other => panic!("expected RateLimited, got {other:?}"),
        }
    }

    #[test]
    fn classify_503_is_server_error() {
        let e = classify_http_error(503, &HeaderMap::new(), "down".into());
        assert!(matches!(e, ProviderError::ServerError { status: 503 }));
    }

    #[test]
    fn classify_400_is_http() {
        let e = classify_http_error(400, &HeaderMap::new(), "bad request".into());
        match e {
            ProviderError::Http { status, body } => {
                assert_eq!(status, 400);
                assert_eq!(body, "bad request");
            }
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[test]
    fn is_retryable_covers_rate_server() {
        assert!(is_retryable(&ProviderError::RateLimited {
            retry_after: Some(Duration::from_secs(1))
        }));
        assert!(is_retryable(&ProviderError::RateLimited {
            retry_after: None
        }));
        assert!(is_retryable(&ProviderError::ServerError { status: 503 }));
        // `Network(reqwest::Error)` cannot be constructed trivially
        // without a real reqwest call; the `matches!` arm in
        // `is_retryable` covers the variant regardless of inner payload.
    }

    #[test]
    fn is_retryable_excludes_auth_http_cost_malformed_canceled() {
        assert!(!is_retryable(&ProviderError::Auth {
            reason: AuthErrorKind::Invalid
        }));
        assert!(!is_retryable(&ProviderError::Http {
            status: 402,
            body: String::new(),
        }));
        assert!(!is_retryable(&ProviderError::CostCapExceeded {
            cap_usd: 1.0,
            spent_usd: 2.0,
        }));
        assert!(!is_retryable(&ProviderError::ToolCallMalformed {
            id: "x".into(),
            error: "y".into(),
        }));
        assert!(!is_retryable(&ProviderError::Canceled));
    }
}
