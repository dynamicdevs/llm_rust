use std::error::Error;

use async_trait::async_trait;

use crate::{prompt::TemplateArgs, schemas::chain::ChainResponse};

#[async_trait]
pub trait ChainTrait: Send + Sync {
    async fn run(&self, input: &dyn TemplateArgs) -> Result<ChainResponse, Box<dyn Error>>;
}
