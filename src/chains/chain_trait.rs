use std::collections::HashMap;

use async_trait::async_trait;

use crate::errors::ApiError;

pub trait InputType {}
impl InputType for HashMap<String, String> {}
impl InputType for String {}

#[async_trait]
pub trait ChainTrait<T: InputType>: Send + Sync {
    async fn run(&mut self, inputs: T) -> Result<String, ApiError>;
}
