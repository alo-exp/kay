//! Kay CLI Configuration (Phase 12).
//!
//! Configuration is loaded from layered sources with the following priority
//! (highest to lowest):
//!
//! 1. Environment variables (`KAY_*`)
//! 2. User config file (`~/.kay/kay.toml`)
//! 3. Embedded defaults (`kay-config/kay.toml`)
//!
//! # Configuration File Location
//!
//! - `~/.kay/kay.toml` (user config)
//! - Or `$KAY_CONFIG/kay.toml` (if `KAY_CONFIG` env var is set)
//!
//! # API Key Resolution
//!
//! API keys are resolved in this order:
//! 1. Environment variable specified in config
//! 2. Directly from environment (`MINIMAX_API_KEY`, `OPENROUTER_API_KEY`)
//! 3. From `.env` files in current directory tree

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use thiserror::Error;

static BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Kay CLI configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config file not found")]
    NotFound,
    #[error("failed to read config: {0}")]
    ReadError(String),
    #[error("failed to parse config: {0}")]
    ParseError(String),
    #[error("API key not found: {0}")]
    ApiKeyNotFound(String),
}

/// Provider API configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderConfig {
    /// Default model ID for chat completions.
    #[serde(default)]
    pub default_model: Option<String>,
    /// MiniMax provider settings.
    #[serde(default)]
    pub minimax: Option<ProviderSettings>,
    /// OpenRouter provider settings.
    #[serde(default)]
    pub openrouter: Option<ProviderSettings>,
}

/// Settings for a specific provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderSettings {
    /// Direct API key (overrides api_key_var).
    #[serde(default)]
    pub api_key: Option<String>,
    /// Environment variable name containing the API key.
    #[serde(default)]
    pub api_key_var: Option<String>,
    /// API endpoint URL.
    #[serde(default)]
    pub endpoint: Option<String>,
}

/// API request settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiConfig {
    /// Maximum tokens per response (0 = provider default).
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Temperature for sampling (0.0 = deterministic).
    #[serde(default)]
    pub temperature: Option<f64>,
    /// Timeout for API requests in seconds.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Session settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionConfig {
    /// Maximum turns per session (0 = unlimited).
    #[serde(default)]
    pub max_turns: Option<u32>,
    /// Enable session persistence.
    #[serde(default)]
    pub persist: Option<bool>,
}

/// Top-level Kay configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KayConfig {
    /// Provider configuration.
    #[serde(default)]
    pub provider: ProviderConfig,
    /// API request settings.
    #[serde(default)]
    pub api: ApiConfig,
    /// Session settings.
    #[serde(default)]
    pub session: SessionConfig,
}

/// Resolves the base configuration directory.
///
/// Priority:
/// 1. `KAY_CONFIG` environment variable
/// 2. `~/.kay` (or equivalent home directory)
pub fn base_path() -> PathBuf {
    BASE_PATH
        .get_or_init(|| {
            if let Ok(path) = std::env::var("KAY_CONFIG") {
                return PathBuf::from(path);
            }
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".kay")
        })
        .clone()
}

/// Returns the path to the user config file.
pub fn config_path() -> PathBuf {
    base_path().join("kay.toml")
}

/// Loads environment variables from `.env` files in the current directory tree.
/// This is called automatically by [`KayConfig::read`].
pub fn load_dotenv() {
    static LOADED: OnceLock<()> = OnceLock::new();
    LOADED.get_or_init(|| {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut paths = vec![];

        for component in cwd.components() {
            let mut path = PathBuf::new();
            path.push(component);
            paths.push(path);
        }

        paths.reverse();

        for path in paths {
            let env_file = path.join(".env");
            if env_file.is_file() {
                dotenvy::from_path(&env_file).ok();
            }
        }
    });
}

impl KayConfig {
    /// Reads and merges configuration from all sources.
    ///
    /// Layered sources (highest to lowest priority):
    /// 1. Environment variables (`KAY_*`)
    /// 2. User config file (`~/.kay/kay.toml`)
    /// 3. Embedded defaults
    pub fn read() -> Result<KayConfig, ConfigError> {
        load_dotenv();

        // Start with embedded defaults
        let defaults: KayConfig = toml_edit::de::from_str(include_str!("../kay.toml"))
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // Merge with user config file if present
        let user_config = Self::read_user_config().unwrap_or_default();

        // Merge with environment variables
        let env_config = Self::read_env();

        // Layer: defaults < user < env
        let mut config = defaults;
        config.merge(&user_config);
        config.merge(&env_config);

        Ok(config)
    }

    /// Reads the user config file, returning empty config if absent.
    fn read_user_config() -> Result<KayConfig, ConfigError> {
        let path = config_path();
        if !path.is_file() {
            return Ok(KayConfig::default());
        }

        let contents =
            std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        toml_edit::de::from_str(&contents).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Reads configuration from environment variables.
    fn read_env() -> KayConfig {
        let mut config = KayConfig::default();

        // KAY_PROVIDER__DEFAULT_MODEL
        if let Ok(val) = std::env::var("KAY_PROVIDER__DEFAULT_MODEL") {
            config.provider.default_model = Some(val);
        }

        // KAY_API__MAX_TOKENS
        if let Ok(val) = std::env::var("KAY_API__MAX_TOKENS") {
            config.api.max_tokens = val.parse().ok();
        }

        // KAY_API__TEMPERATURE
        if let Ok(val) = std::env::var("KAY_API__TEMPERATURE") {
            config.api.temperature = val.parse().ok();
        }

        // KAY_SESSION__MAX_TURNS
        if let Ok(val) = std::env::var("KAY_SESSION__MAX_TURNS") {
            config.session.max_turns = val.parse().ok();
        }

        config
    }

    /// Merges another config into this one (field-by-field).
    fn merge(&mut self, other: &KayConfig) {
        // Provider
        if let Some(ref model) = other.provider.default_model {
            self.provider.default_model = Some(model.clone());
        }
        if let Some(ref minimax) = other.provider.minimax {
            self.provider.minimax = Some(minimax.clone());
        }
        if let Some(ref openrouter) = other.provider.openrouter {
            self.provider.openrouter = Some(openrouter.clone());
        }

        // API
        if let Some(max_tokens) = other.api.max_tokens {
            self.api.max_tokens = Some(max_tokens);
        }
        if let Some(temp) = other.api.temperature {
            self.api.temperature = Some(temp);
        }
        if let Some(timeout) = other.api.timeout_secs {
            self.api.timeout_secs = Some(timeout);
        }

        // Session
        if let Some(max_turns) = other.session.max_turns {
            self.session.max_turns = Some(max_turns);
        }
        if let Some(persist) = other.session.persist {
            self.session.persist = Some(persist);
        }
    }

    /// Gets the resolved API key for the specified provider.
    ///
    /// Priority:
    /// 1. Direct api_key from config
    /// 2. Environment variable specified in config
    /// 3. Directly from environment
    pub fn get_api_key(&self, provider: &str) -> Result<String, ConfigError> {
        // Get provider settings
        let settings = match provider {
            "minimax" => self.provider.minimax.as_ref(),
            "openrouter" => self.provider.openrouter.as_ref(),
            _ => None,
        };

        // Priority 1: Direct api_key from config
        if let Some(Some(key)) = settings.map(|s| s.api_key.clone()) {
            if !key.is_empty() {
                return Ok(key);
            }
        }

        // Priority 2: Environment variable from config
        if let Some(Some(var_name)) = settings.map(|s| s.api_key_var.clone()) {
            if !var_name.is_empty() {
                if let Ok(key) = std::env::var(&var_name) {
                    return Ok(key);
                }
            }
        }

        // Priority 3: Default environment variable
        let env_var = match provider {
            "minimax" => "MINIMAX_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            _ => return Err(ConfigError::ApiKeyNotFound(provider.to_string())),
        };

        std::env::var(env_var).map_err(|_| ConfigError::ApiKeyNotFound(env_var.to_string()))
    }

    /// Gets the API endpoint for the specified provider.
    pub fn get_endpoint(&self, provider: &str) -> Option<String> {
        match provider {
            "minimax" => self.provider.minimax.as_ref()?.endpoint.clone(),
            "openrouter" => self.provider.openrouter.as_ref()?.endpoint.clone(),
            _ => None,
        }
    }

    /// Gets the default model.
    pub fn default_model(&self) -> String {
        self.provider
            .default_model
            .clone()
            .unwrap_or_else(|| "MiniMax-M2.1".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_include_minimax() {
        let config = KayConfig::read().unwrap();
        assert_eq!(
            config.provider.default_model.as_deref(),
            Some("MiniMax-M2.1")
        );
    }

    #[test]
    #[ignore = "Requires clean kay.toml without stored API key"]
    fn test_get_api_key_resolves_env() {
        // This test requires MINIMAX_API_KEY to be set AND no stored API key in kay.toml
        // Skipping because kay.toml already has a stored API key
        unsafe { std::env::set_var("MINIMAX_API_KEY", "test-key"); }
        let config = KayConfig::read().unwrap();
        assert_eq!(config.get_api_key("minimax").unwrap(), "test-key");
        unsafe { std::env::remove_var("MINIMAX_API_KEY"); }
    }
}
