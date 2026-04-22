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
    // W-3 RED: 2-arg signature does NOT match the current 1-arg trait → compile error.
    // W-3 GREEN: trait signature expanded to 2 args, making this compile.
    async fn verify(&self, _task_summary: &str, _task_context: &str) -> VerificationOutcome {
        // TODO: implement in W-3 GREEN
        todo!("MultiPerspectiveVerifier not yet implemented")
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
                .endpoint("http://localhost:9999".into())
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
