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

#[cfg(test)]
pub struct FakeEmbedder {
    pub dimensions: usize,
}

#[cfg(test)]
#[async_trait]
impl EmbeddingProvider for FakeEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, ContextError> {
        Ok(vec![0.0f32; self.dimensions])
    }
}
