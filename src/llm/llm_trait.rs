use async_trait::async_trait;

use crate::errors::ApiError;

#[async_trait]
pub trait LLM {
    async fn generate_completition(&mut self, text: &str) -> Result<String, ApiError>;
}
