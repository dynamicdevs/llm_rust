use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    chat_models::chat_model_trait::ChatTrait,
    errors::ApiError,
    schemas::{
        memory::BaseChatMessageHistory,
        messages::BaseMessage,
        prompt::{BasePromptValue, PromptData},
    },
};

use super::chain_trait::ChainTrait;

pub struct LLMChain<'a> {
    prompt: Box<dyn BasePromptValue>,
    llm: Box<dyn ChatTrait>,
    pub memory: Option<&'a mut dyn BaseChatMessageHistory>,
}

impl<'a> LLMChain<'a> {
    pub fn new(prompt: Box<dyn BasePromptValue>, llm: Box<dyn ChatTrait>) -> Self {
        Self {
            prompt,
            llm,
            memory: None,
        }
    }

    pub fn with_memory(mut self, memory: &'a mut dyn BaseChatMessageHistory) -> Self {
        self.memory = Some(memory);
        self
    }
}

#[async_trait]
impl<'a> ChainTrait<HashMap<String, String>> for LLMChain<'a> {
    async fn run(&mut self, inputs: HashMap<String, String>) -> Result<String, ApiError> {
        self.prompt.add_values(PromptData::HashMapData(inputs));
        let memory_messages = self
            .memory
            .as_ref()
            .map_or(Vec::new(), |memory| memory.messages());

        let prompt_messages = self
            .prompt
            .to_chat_messages()
            .map_err(|e| ApiError::PromptError(e))?;

        let ai_response = self
            .llm
            .generate(vec![memory_messages, prompt_messages.clone()])
            .await?;

        match self.memory.as_mut() {
            Some(memory) => {
                prompt_messages.iter().for_each(|message| {
                    if message.get_type() != "system".to_string() {
                        log::debug!("Adding to memory:{:?}", message.get_content());
                        memory.add_message(message.clone());
                    }
                });
                memory.add_message(Box::new(ai_response.clone()));
            }
            None => {}
        }

        Ok(ai_response.get_content())
    }
}
#[async_trait]
impl<'a> ChainTrait<String> for LLMChain<'a> {
    async fn run(&mut self, inputs: String) -> Result<String, ApiError> {
        self.prompt.add_values(PromptData::VecData(vec![inputs]));
        let memory_messages = self
            .memory
            .as_ref()
            .map_or(Vec::new(), |memory| memory.messages());
        let prompt_messages = self
            .prompt
            .to_chat_messages()
            .map_err(|e| ApiError::PromptError(e))?;

        let ai_response = self
            .llm
            .generate(vec![prompt_messages.clone(), memory_messages])
            .await?;

        match self.memory.as_mut() {
            Some(memory) => {
                prompt_messages.iter().for_each(|message| {
                    if message.get_type() == "human".to_string() {
                        memory.add_message(message.clone());
                    }
                });
                memory.add_message(Box::new(ai_response.clone()));
            }
            None => {}
        }
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
