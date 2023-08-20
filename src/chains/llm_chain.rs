use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    chat_models::chat_model_trait::ChatTrait,
    errors::ApiError,
    schemas::{
        messages::BaseMessage,
        prompt::{BasePromptValue, PromptData},
    },
};

use super::chain_trait::ChainTrait;

//tengo que creatr el prompt
pub struct LLMChain {
    prompt: Box<dyn BasePromptValue>,
    llm: Box<dyn ChatTrait>,
}
impl LLMChain {
    pub fn new(prompt: Box<dyn BasePromptValue>, llm: Box<dyn ChatTrait>) -> Self {
        Self { prompt, llm }
    }
}

#[async_trait]
impl ChainTrait<HashMap<String, String>> for LLMChain {
    async fn run(&mut self, inputs: HashMap<String, String>) -> Result<String, ApiError> {
        self.prompt.add_values(PromptData::HashMapData(inputs));
        let messages = self
            .prompt
            .to_chat_messages()
            .map_err(|e| ApiError::PromptError(e))?;

        let ai_response = self.llm.generate(vec![messages]).await?;
        Ok(ai_response.get_content())
    }
}
#[async_trait]
impl ChainTrait<String> for LLMChain {
    async fn run(&mut self, inputs: String) -> Result<String, ApiError> {
        self.prompt.add_values(PromptData::VecData(vec![inputs]));
        let messages = self
            .prompt
            .to_chat_messages()
            .map_err(|e| ApiError::PromptError(e))?;

        let ai_response = self.llm.generate(vec![messages]).await?;
        Ok(ai_response.get_content())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chains::llm_chain::LLMChain, chat_models::openai::chat_llm::ChatOpenAI,
        prompt::prompt::PromptTemplate,
    };

    use super::*;

    #[tokio::test]
    async fn test_llmchain_run_with_string() {
        let chat_openai = ChatOpenAI::default();
        let prompt_template = PromptTemplate::new("Hola mi nombre es {{name}}.");
        let mut llm_chain = LLMChain::new(Box::new(prompt_template), Box::new(chat_openai));
        let result = llm_chain.run("luis".to_string()).await;
        assert!(result.is_ok());
    }
}
