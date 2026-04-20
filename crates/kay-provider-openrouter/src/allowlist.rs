//! Model allowlist gate (PROV-04, D-07, NN-6, AC-07).
//!
//! Responsibilities:
//!   - Parse a canonical list of allowed models from a JSON config file OR
//!     from the `KAY_ALLOWED_MODELS` env var (comma-separated).
//!   - Normalize both the allowlist entries and caller-supplied requests to
//!     a canonical form: ascii-lowercase, trimmed, no `:exacto` suffix
//!     (Pitfall 7).
//!   - Reject model IDs containing control characters (`\r \n \t`) or
//!     non-ASCII bytes BEFORE the allowlist compare (TM-04).
//!   - Rewrite the on-wire model ID to always include `:exacto` (Pitfall 8,
//!     TM-08).
//!
//! Failure surface: `ProviderError::ModelNotAllowlisted` for all rejection
//! paths (unknown model AND invalid charset).

use std::path::Path;

use serde::Deserialize;

use crate::error::ProviderError;

const ENV_OVERRIDE: &str = "KAY_ALLOWED_MODELS";

#[derive(Debug, Clone)]
pub struct Allowlist {
    /// Canonical form: lowercase, trimmed, no `:exacto` suffix.
    models: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ConfigFileShape {
    /// The rest of the file (id, api_key_vars, url, ...) is ignored via
    /// serde's default behavior — we only extract `allowed_models`.
    #[serde(default)]
    allowed_models: Vec<String>,
}

impl Allowlist {
    /// Load allowlist from a JSON config file. The file may contain additional
    /// keys (id, api_key_vars, url, etc.); only `allowed_models` is extracted.
    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let shape: ConfigFileShape = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self::from_models(shape.allowed_models))
    }

    /// Construct directly from a Vec<String>. Canonicalizes each entry.
    /// Empty lists are allowed (every request will be rejected — useful for
    /// testing the deny-all case).
    pub fn from_models(models: Vec<String>) -> Self {
        let models = models
            .into_iter()
            .map(|m| Self::canonicalize(&m))
            .filter(|m| !m.is_empty())
            .collect();
        Self { models }
    }

    /// Apply the `KAY_ALLOWED_MODELS` env override if set.
    /// Env format: comma-separated canonical model IDs.
    /// If the env var is unset OR empty, returns self unchanged.
    pub fn with_env_override(self) -> Self {
        match std::env::var(ENV_OVERRIDE) {
            Ok(env) if !env.trim().is_empty() => {
                let models = env
                    .split(',')
                    .map(Self::canonicalize)
                    .filter(|m| !m.is_empty())
                    .collect();
                Self { models }
            }
            _ => self,
        }
    }

    /// Validate a caller-supplied model ID against the allowlist.
    /// Rejects control-char or non-ASCII IDs with the same error variant
    /// (TM-04: a caller smuggling `\r\n` into a model ID gets the same
    /// treatment as an unknown model).
    pub fn check(&self, requested: &str) -> Result<(), ProviderError> {
        Self::validate_charset(requested)?;
        let canonical = Self::canonicalize(requested);
        if self.models.iter().any(|m| m == &canonical) {
            Ok(())
        } else {
            Err(ProviderError::ModelNotAllowlisted {
                requested: requested.to_string(),
                allowed: self.models.clone(),
            })
        }
    }

    /// Return the on-wire model ID. Always appends `:exacto` (Pitfall 8).
    /// Expects `canonical` to already be in canonical form (the caller
    /// should invoke after `check()` has succeeded).
    pub fn to_wire_model(&self, canonical: &str) -> String {
        format!("{}:exacto", Self::canonicalize(canonical))
    }

    /// Canonical model list (used by `Provider::models()`).
    pub fn models(&self) -> &[String] {
        &self.models
    }

    // --- Private helpers ---

    fn canonicalize(m: &str) -> String {
        m.trim()
            .to_ascii_lowercase()
            .trim_end_matches(":exacto")
            .to_string()
    }

    fn validate_charset(m: &str) -> Result<(), ProviderError> {
        let bad = m.contains(['\r', '\n', '\t']) || !m.is_ascii();
        if bad {
            Err(ProviderError::ModelNotAllowlisted {
                requested: m.to_string(),
                allowed: Vec::new(), // Intentional: don't leak the allowlist
                                     // to a caller that tried to smuggle
                                     // control chars. They get an empty list.
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use std::sync::Mutex;

    // Serialize env-mutating tests. Rust's test harness parallelizes within a
    // binary by default; std::env mutation is process-global. Without this
    // guard, `env_override_*` tests race and cause spurious failures.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn canonicalize_lowercases_trims_strips_exacto() {
        assert_eq!(
            Allowlist::canonicalize("  Anthropic/Claude-Sonnet-4.6:exacto  "),
            "anthropic/claude-sonnet-4.6"
        );
    }

    #[test]
    fn check_accepts_canonical_and_mixed_case() {
        let a = Allowlist::from_models(vec!["anthropic/claude-sonnet-4.6".into()]);
        assert!(a.check("anthropic/claude-sonnet-4.6").is_ok());
        assert!(a.check("Anthropic/Claude-Sonnet-4.6").is_ok());
        assert!(a.check("anthropic/claude-sonnet-4.6:exacto").is_ok());
    }

    #[test]
    fn check_rejects_unknown_model() {
        let a = Allowlist::from_models(vec!["anthropic/claude-sonnet-4.6".into()]);
        match a.check("openai/gpt-5.4") {
            Err(ProviderError::ModelNotAllowlisted { requested, allowed }) => {
                assert_eq!(requested, "openai/gpt-5.4");
                assert_eq!(allowed, vec!["anthropic/claude-sonnet-4.6".to_string()]);
            }
            other => panic!("expected ModelNotAllowlisted, got {other:?}"),
        }
    }

    #[test]
    fn check_rejects_crlf_and_tabs() {
        let a = Allowlist::from_models(vec!["anthropic/claude-sonnet-4.6".into()]);
        for bad in &["foo\r", "foo\n", "foo\t"] {
            let r = a.check(bad);
            assert!(
                matches!(r, Err(ProviderError::ModelNotAllowlisted { .. })),
                "expected reject for {bad:?}, got {r:?}"
            );
            // Allowed list empty on charset rejection (see validate_charset comment)
            if let Err(ProviderError::ModelNotAllowlisted { allowed, .. }) = r {
                assert!(allowed.is_empty());
            }
        }
    }

    #[test]
    fn check_rejects_non_ascii() {
        let a = Allowlist::from_models(vec!["anthropic/claude-sonnet-4.6".into()]);
        assert!(matches!(
            a.check("anthropic/clàude"),
            Err(ProviderError::ModelNotAllowlisted { .. })
        ));
    }

    #[test]
    fn to_wire_model_always_appends_exacto() {
        let a = Allowlist::from_models(vec!["anthropic/claude-sonnet-4.6".into()]);
        assert_eq!(
            a.to_wire_model("anthropic/claude-sonnet-4.6"),
            "anthropic/claude-sonnet-4.6:exacto"
        );
        // Already-suffixed input is idempotent
        assert_eq!(
            a.to_wire_model("anthropic/claude-sonnet-4.6:exacto"),
            "anthropic/claude-sonnet-4.6:exacto"
        );
    }

    #[test]
    fn env_override_replaces_base_list() {
        // SAFETY: env mutation is process-global; ENV_LOCK serializes these
        // tests within the unit-test binary. Rust 2024 marks env mutation as
        // unsafe due to cross-thread data-race potential; we accept that
        // for test-only scope.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var(ENV_OVERRIDE, "fake/model-a, fake/model-b");
        }
        let a =
            Allowlist::from_models(vec!["anthropic/claude-sonnet-4.6".into()]).with_env_override();
        assert!(a.check("fake/model-a").is_ok());
        assert!(a.check("fake/model-b").is_ok());
        assert!(a.check("anthropic/claude-sonnet-4.6").is_err());
        unsafe {
            std::env::remove_var(ENV_OVERRIDE);
        }
    }

    #[test]
    fn env_override_empty_string_leaves_base_list_alone() {
        // SAFETY: see above.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var(ENV_OVERRIDE, "");
        }
        let a =
            Allowlist::from_models(vec!["anthropic/claude-sonnet-4.6".into()]).with_env_override();
        assert!(a.check("anthropic/claude-sonnet-4.6").is_ok());
        unsafe {
            std::env::remove_var(ENV_OVERRIDE);
        }
    }
}
