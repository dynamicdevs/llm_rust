use async_trait::async_trait;

use crate::errors::ApiError;

use super::schemas::{AIMessage, BaseMessage};

#[async_trait]
pub trait ChatTrait {
    async fn generate(
        &self,
        messages: Vec<Vec<Box<dyn BaseMessage>>>,
    ) -> Result<AIMessage, ApiError>;
}
