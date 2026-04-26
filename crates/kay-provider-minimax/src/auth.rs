//! API key resolution for MiniMax.
//!
//! Resolution order:
//!   1. `MINIMAX_API_KEY` env var (Kay's primary)
//!   2. `config.api_key` from ConfigAuthSource (if provided and non-empty)
//!   3. Else `ProviderError::Auth { reason: AuthErrorKind::Missing }`.

use crate::error::{AuthErrorKind, ProviderError};

const ENV_MINIMAX_API_KEY: &str = "MINIMAX_API_KEY";

/// Opaque API key. The inner String is NEVER surfaced via Debug/Display.
pub struct ApiKey(String);

impl ApiKey {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ApiKey {
    fn from(s: String) -> Self {
        Self(s.trim().to_string())
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

/// Config-file source for auth.
#[derive(Debug, Default, Clone)]
pub struct ConfigAuthSource {
    pub api_key: Option<String>,
}

impl ConfigAuthSource {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

/// Resolve the API key per precedence rules.
pub fn resolve_api_key(config: Option<&ConfigAuthSource>) -> Result<ApiKey, ProviderError> {
    // 1. MINIMAX_API_KEY env var (Kay's primary).
    if let Ok(env_val) = std::env::var(ENV_MINIMAX_API_KEY) {
        let trimmed = env_val.trim();
        if !trimmed.is_empty() {
            return Ok(ApiKey(trimmed.to_string()));
        }
    }

    // 2. Config file fallback.
    if let Some(src) = config && let Some(ref key) = src.api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return Ok(ApiKey(trimmed.to_string()));
        }
    }

    // 3. Nothing found -> typed error.
    Err(ProviderError::Auth { reason: AuthErrorKind::Missing })
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn empty_env_var_treated_as_unset() {
        unsafe {
            std::env::remove_var(ENV_MINIMAX_API_KEY);
            std::env::set_var(ENV_MINIMAX_API_KEY, "   ");
        }
        let cfg = ConfigAuthSource::new(Some("sk-from-config".into()));
        let key = resolve_api_key(Some(&cfg)).expect("should resolve");
        assert_eq!(key.as_str(), "sk-from-config");
        unsafe {
            std::env::remove_var(ENV_MINIMAX_API_KEY);
        }
    }

    #[test]
    fn missing_everywhere_surfaces_typed_error() {
        unsafe {
            std::env::remove_var(ENV_MINIMAX_API_KEY);
        }
        let r = resolve_api_key(None);
        assert!(matches!(r, Err(ProviderError::Auth { reason: AuthErrorKind::Missing })));
    }
}
