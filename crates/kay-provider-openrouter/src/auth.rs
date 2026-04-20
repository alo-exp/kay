//! API key resolution (PROV-03, D-08) and credential-material redaction
//! (TM-01).
//!
//! Resolution order:
//!   1. `OPENROUTER_API_KEY` env var (if non-empty) -> wins.
//!   2. `config.api_key` from ConfigAuthSource (if provided and non-empty).
//!   3. Else `ProviderError::Auth { reason: AuthErrorKind::Missing }`.
//!
//! No OAuth (explicit PROV-03). Phase 10's UI-04 will add keyring-at-rest.

use crate::error::{AuthErrorKind, ProviderError};

const ENV_API_KEY: &str = "OPENROUTER_API_KEY";

/// Opaque API key. The inner String is NEVER surfaced via Debug/Display.
///
/// Use `as_str()` only at the HTTP-call boundary (plan 02-08). Any other
/// crate-internal `.as_str()` call is a code-review fail.
pub struct ApiKey(String);

impl ApiKey {
    /// Crate-internal accessor for constructing the HTTP Authorization
    /// header. Not re-exported through `pub use` in lib.rs.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Constructor for builder-side use (plan 02-08 T1). Trims ambient whitespace
/// so callers can pass raw env/config values without a separate sanitize step.
/// Empty input is allowed here — the upstream HTTP call will surface a 401,
/// mapped to `Auth::Invalid` by the classifier in plan 02-10.
impl From<String> for ApiKey {
    fn from(s: String) -> Self {
        Self(s.trim().to_string())
    }
}

/// TM-01: custom Debug that redacts the inner string. NEVER #[derive(Debug)]
/// on ApiKey — that would trivially leak the key through any error trace.
impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

/// Config-file source for auth. In Phase 2 this is constructed by the
/// caller (e.g., kay-cli Phase 5 will parse kay.toml). Phase 2's own tests
/// construct it inline.
#[derive(Debug, Default, Clone)]
pub struct ConfigAuthSource {
    pub api_key: Option<String>,
}

impl ConfigAuthSource {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

/// Resolve the API key per D-08 precedence rules.
pub fn resolve_api_key(config: Option<&ConfigAuthSource>) -> Result<ApiKey, ProviderError> {
    // 1. Env var wins if non-empty.
    if let Ok(env_val) = std::env::var(ENV_API_KEY) {
        let trimmed = env_val.trim();
        if !trimmed.is_empty() {
            return Ok(ApiKey(trimmed.to_string()));
        }
    }

    // 2. Config file fallback.
    if let Some(src) = config
        && let Some(ref key) = src.api_key
    {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return Ok(ApiKey(trimmed.to_string()));
        }
    }

    // 3. Nothing found -> typed error.
    Err(ProviderError::Auth { reason: AuthErrorKind::Missing })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod unit {
    use super::*;
    use std::sync::Mutex;

    // Serialize env-mutating tests. Rust's test harness parallelizes within
    // a binary by default; std::env mutation is process-global.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn env_wins_over_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: env mutation is process-global; ENV_LOCK serializes.
        unsafe {
            std::env::remove_var(ENV_API_KEY);
            std::env::set_var(ENV_API_KEY, "sk-from-env");
        }
        let cfg = ConfigAuthSource::new(Some("sk-from-config".into()));
        let key = resolve_api_key(Some(&cfg)).expect("should resolve");
        assert_eq!(key.as_str(), "sk-from-env");
        unsafe {
            std::env::remove_var(ENV_API_KEY);
        }
    }

    #[test]
    fn config_used_when_env_unset() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var(ENV_API_KEY);
        }
        let cfg = ConfigAuthSource::new(Some("sk-from-config".into()));
        let key = resolve_api_key(Some(&cfg)).expect("should resolve");
        assert_eq!(key.as_str(), "sk-from-config");
    }

    #[test]
    fn empty_env_var_treated_as_unset() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var(ENV_API_KEY);
            std::env::set_var(ENV_API_KEY, "   ");
        }
        let cfg = ConfigAuthSource::new(Some("sk-from-config".into()));
        let key = resolve_api_key(Some(&cfg)).expect("should resolve");
        assert_eq!(key.as_str(), "sk-from-config");
        unsafe {
            std::env::remove_var(ENV_API_KEY);
        }
    }

    #[test]
    fn missing_everywhere_surfaces_typed_error() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var(ENV_API_KEY);
        }
        let r = resolve_api_key(None);
        assert!(matches!(
            r,
            Err(ProviderError::Auth { reason: AuthErrorKind::Missing })
        ));
    }

    #[test]
    fn empty_config_and_missing_env_is_missing() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var(ENV_API_KEY);
        }
        let cfg = ConfigAuthSource::new(Some("".into()));
        let r = resolve_api_key(Some(&cfg));
        assert!(matches!(
            r,
            Err(ProviderError::Auth { reason: AuthErrorKind::Missing })
        ));
    }

    #[test]
    fn debug_redacts_key_material() {
        let k = ApiKey("sk-super-secret-value".to_string());
        let rendered = format!("{k:?}");
        assert_eq!(rendered, "ApiKey(<redacted>)");
        assert!(!rendered.contains("sk-super"));
        assert!(!rendered.contains("secret"));
    }
}
