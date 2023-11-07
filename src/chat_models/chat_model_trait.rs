use async_trait::async_trait;

use crate::{
    errors::ApiError,
    schemas::{llm::LlmResponse, messages::BaseMessage},
};

#[async_trait]
pub trait ChatTrait: Send + Sync {
    async fn generate(
        &self,
        messages: Vec<Vec<Box<dyn BaseMessage>>>,
    ) -> Result<LlmResponse, ApiError>;
}
