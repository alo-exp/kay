use crate::error::ContextError;
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ContextError>;
}

pub struct NoOpEmbedder;

#[async_trait]
impl EmbeddingProvider for NoOpEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, ContextError> {
        Ok(vec![])
    }
}

/// Deterministic test double — returns zero-vectors of length `dimensions`.
/// Always compiled so integration tests (tests/) can import it without feature flags.
pub struct FakeEmbedder {
    pub dimensions: usize,
}

#[async_trait]
impl EmbeddingProvider for FakeEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, ContextError> {
        Ok(vec![0.0f32; self.dimensions])
    }
}

impl FakeEmbedder {
    pub fn embed_sync(&self, _text: &str) -> Vec<f32> {
        vec![0.0f32; self.dimensions]
    }
}
