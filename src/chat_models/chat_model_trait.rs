use async_trait::async_trait;

use crate::errors::ApiError;

use super::schemas::{AIMessage, BaseMessage};

#[async_trait]
pub trait ChatTrait: Send + Sync {
    async fn generate(
        &self,
        messages: Vec<Vec<Box<dyn BaseMessage>>>,
    ) -> Result<AIMessage, ApiError>;

    async fn call(&self, query: String) -> Result<String, ApiError>;
}
