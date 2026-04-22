use crate::budget::{ContextBudget, ContextPacket};
use crate::error::ContextError;
use async_trait::async_trait;

/// Phase 8+ implementors: if session.title is injected into the system prompt,
/// it MUST be delimited as [USER_DATA: session_title] per Phase 6 DL-7.
#[async_trait]
pub trait ContextEngine: Send + Sync {
    async fn retrieve(
        &self,
        prompt: &str,
        schemas: &[serde_json::Value],
    ) -> Result<ContextPacket, ContextError>;
}

pub struct NoOpContextEngine;

impl Default for NoOpContextEngine {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl ContextEngine for NoOpContextEngine {
    async fn retrieve(
        &self,
        _prompt: &str,
        _schemas: &[serde_json::Value],
    ) -> Result<ContextPacket, ContextError> {
        Ok(ContextPacket::default())
    }
}

pub struct KayContextEngine {
    pub budget: ContextBudget,
}
