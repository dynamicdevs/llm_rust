use async_trait::async_trait;

use crate::errors::ApiError;

#[async_trait]
pub trait BaseLLM: Send + Sync {
    async fn generate(&self, prompt: String) -> Result<String, ApiError>;
}
