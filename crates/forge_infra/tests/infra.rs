use forge_infra::sanitize_headers;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

#[test]
fn sanitize_headers_redacts_authorization() {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer secret-api-key"),
    );
    headers.insert(
        "x-api-key",
        HeaderValue::from_static("another-secret"),
    );
    headers.insert(
        "content-type",
        HeaderValue::from_static("application/json"),
    );

    let sanitized = sanitize_headers(&headers);

    // Authorization headers must be redacted
    assert_eq!(
        sanitized.get("authorization"),
        Some(&HeaderValue::from_static("[REDACTED]")),
    );
    assert_eq!(
        sanitized.get("x-api-key"),
        Some(&HeaderValue::from_static("[REDACTED]")),
    );
    // Non-sensitive headers must be preserved
    assert_eq!(
        sanitized.get("content-type"),
        Some(&HeaderValue::from_static("application/json")),
    );
}
