use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kay_provider_openrouter::{CostCap, OpenRouterProvider};
use kay_tools::{
    events::AgentEvent,
    seams::verifier::{TaskVerifier, VerificationOutcome},
};

use crate::mode::{VerifierConfig, VerifierMode};

pub struct MultiPerspectiveVerifier {
    provider: Arc<OpenRouterProvider>,
    cost_cap: Arc<CostCap>,
    config: VerifierConfig,
    verifier_cost: Arc<Mutex<f64>>,
    stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
}

impl MultiPerspectiveVerifier {
    pub fn new(
        provider: Arc<OpenRouterProvider>,
        cost_cap: Arc<CostCap>,
        config: VerifierConfig,
        stream_sink: Arc<dyn Fn(AgentEvent) + Send + Sync>,
    ) -> Self {
        Self {
            provider,
            cost_cap,
            config,
            verifier_cost: Arc::new(Mutex::new(0.0)),
            stream_sink,
        }
    }
}

#[async_trait]
impl TaskVerifier for MultiPerspectiveVerifier {
    async fn verify(&self, task_summary: &str, task_context: &str) -> VerificationOutcome {
        if matches!(self.config.mode, VerifierMode::Disabled) {
            return VerificationOutcome::Pass {
                note: "verifier disabled (VerifierMode::Disabled)".into(),
            };
        }
        // W-3: Disabled path implemented. Full Interactive/Benchmark critic
        // calls wired in W-6 once cost ceiling + event ordering are in place.
        // For now, return Pass so the agent loop can make forward progress.
        let _ = (task_summary, task_context);
        VerificationOutcome::Pass {
            note: "stub: critic calls wired in W-6".into(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn noop_sink() -> Arc<dyn Fn(AgentEvent) + Send + Sync> {
        Arc::new(|_ev: AgentEvent| {})
    }

    fn make_verifier(mode: VerifierMode) -> MultiPerspectiveVerifier {
        let provider = Arc::new(
            OpenRouterProvider::builder()
                .endpoint("http://localhost:9999".to_string())
                .api_key("test-key-not-used")
                .build()
                .expect("builder"),
        );
        let cost_cap = Arc::new(CostCap::uncapped());
        let config = VerifierConfig { mode, ..VerifierConfig::default() };
        MultiPerspectiveVerifier::new(provider, cost_cap, config, noop_sink())
    }

    #[tokio::test]
    async fn disabled_mode_returns_pass_immediately() {
        let v = make_verifier(VerifierMode::Disabled);
        let outcome = v.verify("summary", "ctx").await;
        assert!(
            matches!(outcome, VerificationOutcome::Pass { .. }),
            "Disabled mode must return Pass immediately: {outcome:?}"
        );
    }

    #[tokio::test]
    async fn never_returns_pending() {
        // Non-Negotiable #6: MultiPerspectiveVerifier MUST NEVER return Pending
        let v = make_verifier(VerifierMode::Disabled);
        let outcome = v.verify("summary", "ctx").await;
        assert!(
            !matches!(outcome, VerificationOutcome::Pending { .. }),
            "MultiPerspectiveVerifier must NEVER return Pending"
        );
    }

    #[test]
    fn is_dyn_compatible() {
        fn _check(_: &dyn TaskVerifier) {}
    }
}
