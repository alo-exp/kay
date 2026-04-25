// project_settings.rs — Phase 10 project settings (kay-tauri).
// See: docs/superpowers/specs/2026-04-24-phase10-multi-session-manager-design.md
//
// WAVE 4 (RED): ProjectSettings struct stub. Real implementation in GREEN wave.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Project-level settings persisted to `~/.kay/projects/<path>/settings.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProjectSettings {
    /// Absolute path to project root.
    pub project_path: String,
    /// Reference to keychain entry for OpenRouter key.
    pub openrouter_key_alias: Option<String>,
    /// Model tier for the model picker.
    pub model_allowlist_tier: ModelTier,
    /// Verifier policy settings.
    pub verifier_policy: VerifierPolicy,
    /// Sandbox policy settings.
    pub sandbox_policy: SandboxPolicy,
    /// Command approval mode.
    pub command_approval: CommandApproval,
}

/// Model selection tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ModelTier {
    /// Exacto allowlist only (verified safe).
    Recommended,
    /// Smoke-tested models.
    Experimental,
    /// Any model not explicitly allowlisted (shows warning).
    All,
}

/// Verifier policy settings.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VerifierPolicy {
    pub enabled: bool,
    pub max_retries: u32,
    pub cost_ceiling_usd: f64,
}

/// Sandbox policy settings.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SandboxPolicy {
    /// Allowed paths for file operations.
    pub allowed_paths: Vec<String>,
    /// Denied paths for file operations.
    pub denied_paths: Vec<String>,
    /// Allowed network hosts.
    pub net_whitelist: Vec<String>,
}

/// Command approval mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum CommandApproval {
    /// Never show approval dialog.
    Off,
    /// Show on first use of each tool.
    OnFirstUse,
    /// Always show before tool execution.
    Always,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            project_path: String::new(),
            openrouter_key_alias: None,
            model_allowlist_tier: ModelTier::Recommended,
            verifier_policy: VerifierPolicy::default(),
            sandbox_policy: SandboxPolicy::default(),
            command_approval: CommandApproval::Off,
        }
    }
}

impl Default for VerifierPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            cost_ceiling_usd: 10.0,
        }
    }
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            allowed_paths: vec![],
            denied_paths: vec![],
            net_whitelist: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_settings_default_is_valid() {
        let settings = ProjectSettings::default();
        assert_eq!(settings.model_allowlist_tier, ModelTier::Recommended);
        assert!(settings.verifier_policy.enabled);
        assert_eq!(settings.command_approval, CommandApproval::Off);
    }

    #[test]
    fn project_settings_serializes_to_json() {
        let settings = ProjectSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(!json.is_empty());
        let loaded: ProjectSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.model_allowlist_tier, ModelTier::Recommended);
    }
}