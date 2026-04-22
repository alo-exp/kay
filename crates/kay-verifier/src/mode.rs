#[derive(Debug, Clone)]
pub enum VerifierMode {
    /// Single critic (EndUser only) — default for interactive sessions.
    Interactive,
    /// All three critics — for benchmark mode.
    Benchmark,
    /// Bypass all verification — for --no-verify.
    Disabled,
}

#[derive(Debug, Clone)]
pub struct VerifierConfig {
    pub mode: VerifierMode,
    pub max_retries: u32,
    pub cost_ceiling_usd: f64,
    pub model: String,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            mode: VerifierMode::Interactive,
            max_retries: 3,
            cost_ceiling_usd: 1.0,
            model: "openai/gpt-4o-mini".into(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_interactive() {
        let cfg = VerifierConfig::default();
        assert!(matches!(cfg.mode, VerifierMode::Interactive));
    }

    #[test]
    fn default_max_retries_is_3() {
        let cfg = VerifierConfig::default();
        assert_eq!(cfg.max_retries, 3);
    }

    #[test]
    fn default_cost_ceiling_is_1_usd() {
        let cfg = VerifierConfig::default();
        assert!((cfg.cost_ceiling_usd - 1.0).abs() < 1e-9);
    }

    #[test]
    fn default_model_is_gpt4o_mini() {
        let cfg = VerifierConfig::default();
        assert_eq!(cfg.model, "openai/gpt-4o-mini");
    }
}
