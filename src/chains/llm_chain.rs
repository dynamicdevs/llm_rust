use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    chat_models::{chat_model_trait::ChatTrait, openai::chat_llm::ChatModel},
    errors::ApiError,
    prompt::template_type::Prompt,
};

use super::chain_trait::ChainTrait;

pub struct LLMChain {
    prompt: Prompt,
    llm: ChatTrait,
}

#[async_trait]
impl ChainTrait<HashMap<String, String>> for LLMChain {
    async fn run(&self, inputs: HashMap<String, String>) -> Result<String, ApiError> {
        self.llm.generate(messages);
        unimplemented!()
    }
}

#[async_trait]
impl ChainTrait<String> for LLMChain {
    async fn run(&self, inputs: String) -> Result<String, ApiError> {
        // Your actual implementation here for String input
        // Just a dummy return for now
        Ok(format!("Received input: {}", inputs))
    }
}
