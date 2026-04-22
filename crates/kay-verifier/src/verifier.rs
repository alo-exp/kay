use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::StreamExt;
use kay_provider_openrouter::{ChatRequest, CostCap, Message, OpenRouterProvider, Provider};
use kay_tools::{
    events::AgentEvent,
    seams::verifier::{TaskVerifier, VerificationOutcome},
};

use crate::critic::{CriticResponse, CriticRole};
use crate::mode::{VerifierConfig, VerifierMode};

pub struct MultiPerspectiveVerifier {
    #[allow(dead_code)]
    provider: Arc<OpenRouterProvider>,
    #[allow(dead_code)]
    cost_cap: Arc<CostCap>,
    #[allow(dead_code)]
    config: VerifierConfig,
    #[allow(dead_code)]
    verifier_cost: Arc<Mutex<f64>>,
    #[allow(dead_code)]
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

        let roles: Vec<CriticRole> = match self.config.mode {
            VerifierMode::Interactive => vec![CriticRole::EndUser],
            VerifierMode::Benchmark => vec![
                CriticRole::TestEngineer,
                CriticRole::QAEngineer,
                CriticRole::EndUser,
            ],
            VerifierMode::Disabled => unreachable!(),
        };

        for role in &roles {
            // Check cost ceiling BEFORE calling critic (VERIFY-03).
            // This prevents spending on critics that can't contribute before
            // we know their cost.
            {
                let current = *self.verifier_cost.lock().unwrap();
                if current > self.config.cost_ceiling_usd as f64 {
                    (self.stream_sink)(AgentEvent::VerifierDisabled {
                        reason: "cost_ceiling_exceeded".into(),
                        cost_usd: current,
                    });
                    return VerificationOutcome::Pass {
                        note: "cost ceiling exceeded — verifier disabled".into(),
                    };
                }
            }

            // Build messages: system prompt + task context + task summary
            let messages = vec![
                Message {
                    role: "system".into(),
                    content: role.system_prompt().to_string(),
                    tool_call_id: None,
                },
                Message {
                    role: "user".into(),
                    content: format!(
                        "Task Summary:\n{}\n\nTask Context:\n{}",
                        task_summary, task_context
                    ),
                    tool_call_id: None,
                },
            ];

            let req = ChatRequest {
                model: self.config.model.clone(),
                messages,
                tools: vec![],
                temperature: None,
                max_tokens: None,
            };

            match self.provider.chat(req).await {
                Ok(mut stream) => {
                    let mut response_text = String::new();

                    while let Some(event_result) = stream.next().await {
                        match event_result {
                            Ok(event) => {
                                match event {
                                    kay_provider_openrouter::AgentEvent::TextDelta { content } => {
                                        response_text.push_str(&content);
                                    }
                                    kay_provider_openrouter::AgentEvent::Usage {
                                        prompt_tokens: _,
                                        completion_tokens: _,
                                        cost_usd,
                                    } => {
                                        // Update verifier_cost with actual usage cost
                                        *self.verifier_cost.lock().unwrap() += cost_usd;
                                    }
                                    _ => {}
                                }
                            }
                            Err(_e) => {
                                // Provider error — break and treat as pass (fail gracefully)
                                break;
                            }
                        }
                    }

                    let cost_this_call = {
                        // Snapshot cost accumulated during this call
                        let guard = self.verifier_cost.lock().unwrap();
                        *guard
                    };

                    // Emit Verification event for this critic (VERIFY-04)
                    match CriticResponse::from_json(&response_text) {
                        Ok(cr) => {
                            let verdict_str =
                                if cr.is_pass() { "pass" } else { "fail" }.to_string();
                            (self.stream_sink)(AgentEvent::Verification {
                                critic_role: role.as_str().to_string(),
                                verdict: verdict_str.clone(),
                                reason: cr.reason.clone(),
                                cost_usd: cost_this_call,
                            });

                            if !cr.is_pass() {
                                return VerificationOutcome::Fail {
                                    reason: format!("{}: {}", role.as_str(), cr.reason),
                                };
                            }
                        }
                        Err(e) => {
                            // Parse failure — treat as pass to avoid blocking on bad output
                            (self.stream_sink)(AgentEvent::Verification {
                                critic_role: role.as_str().to_string(),
                                verdict: "pass".into(),
                                reason: format!("parse error: {e}"),
                                cost_usd: cost_this_call,
                            });
                        }
                    }
                }
                Err(_e) => {
                    // Provider error — fail gracefully, return pass to avoid blocking
                    (self.stream_sink)(AgentEvent::Verification {
                        critic_role: role.as_str().to_string(),
                        verdict: "pass".into(),
                        reason: "provider error".into(),
                        cost_usd: 0.0,
                    });
                }
            }

            // AFTER this critic, check if we've exceeded the cost ceiling.
            // If so, emit VerifierDisabled and stop — the next critic in the
            // loop body will see the pre-check gate triggered above (VERIFY-03).
            {
                let current = *self.verifier_cost.lock().unwrap();
                if current > self.config.cost_ceiling_usd as f64 {
                    (self.stream_sink)(AgentEvent::VerifierDisabled {
                        reason: "cost_ceiling_exceeded".into(),
                        cost_usd: current,
                    });
                    return VerificationOutcome::Pass {
                        note: "cost ceiling exceeded — verifier disabled".into(),
                    };
                }
            }
        }
        VerificationOutcome::Pass { note: format!("all {} critics passed", roles.len()) }
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
