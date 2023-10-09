use std::error::Error;

use async_trait::async_trait;

use crate::prompt::TemplateArgs;

#[async_trait]
pub trait ChainTrait: Send + Sync {
    async fn run(&self, input: &dyn TemplateArgs) -> Result<String, Box<dyn Error>>;
}
