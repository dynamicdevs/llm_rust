use std::error::Error;

use async_trait::async_trait;

use crate::{
    chat_models::chat_model_trait::ChatTrait,
    errors::ApiError,
    prompt::{BaseChatPromptTemplate, ChatPromptTemplate},
    schemas::{memory::BaseChatMessageHistory, messages::BaseMessage},
};

use super::chain_trait::{ChainInput, ChainTrait};

//Chat Chain
pub struct LLMChatChain<'a> {
    prompt: ChatPromptTemplate,
    header_prompts: Option<Vec<Box<dyn BaseMessage>>>,
    sandwich_prompts: Option<Vec<Box<dyn BaseMessage>>>,
    llm: Box<dyn ChatTrait>,
    pub memory: Option<&'a mut dyn BaseChatMessageHistory>,
}

impl<'a> LLMChatChain<'a> {
    pub fn new(prompt: ChatPromptTemplate, llm: Box<dyn ChatTrait>) -> Self {
        Self {
            prompt,
            llm,
            memory: None,
            header_prompts: None,
            sandwich_prompts: None,
        }
    }

    pub fn with_memory(mut self, memory: &'a mut dyn BaseChatMessageHistory) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn with_header_prompts(mut self, header_prompts: Vec<Box<dyn BaseMessage>>) -> Self {
        self.header_prompts = Some(header_prompts);
        self
    }

    pub fn with_sandwich_prompts(mut self, sandwich_prompts: Vec<Box<dyn BaseMessage>>) -> Self {
        self.sandwich_prompts = Some(sandwich_prompts);
        self
    }

    fn order_messages(
        &self,
        prompt_messages: Vec<Box<dyn BaseMessage>>,
    ) -> Vec<Vec<Box<dyn BaseMessage>>> {
        let mut all_messages: Vec<Vec<Box<dyn BaseMessage>>> = Vec::new();

        if let Some(header) = self.header_prompts.as_ref() {
            all_messages.push(header.clone());
        }

        all_messages.push(
            self.memory
                .as_ref()
                .map_or(Vec::new(), |memory| memory.messages()),
        );

        if let Some(sandwich) = self.sandwich_prompts.as_ref() {
            all_messages.push(sandwich.clone());
        }

        all_messages.push(prompt_messages);

        all_messages
    }

    async fn execute(
        &mut self,
        prompt_messages: Vec<Box<dyn BaseMessage>>,
    ) -> Result<String, ApiError> {
        let all_messages = self.order_messages(prompt_messages.clone());

        let ai_response = self.llm.generate(all_messages).await?;

        if let Some(memory) = self.memory.as_mut() {
            for message in &prompt_messages {
                if message.get_type() == String::from("user") {
                    memory.add_message(message.clone());
                }
            }
            memory.add_message(Box::new(ai_response.clone()));
        }

        Ok(ai_response.get_content())
    }
}

#[async_trait]
impl<'a> ChainTrait for LLMChatChain<'a> {
    async fn run(&mut self, inputs: ChainInput) -> Result<String, Box<dyn Error>> {
        let prompt_value = self.prompt.format_prompt(&inputs)?;
        let prompt_messages = prompt_value.to_chat_messages()?;

        Ok(self
            .execute(prompt_messages)
            .await
            .map_err(|e| Box::new(e))?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chains::llmchat_chain::LLMChatChain,
        chat_models::openai::chat_llm::ChatOpenAI,
        prompt::{HumanMessagePromptTemplate, MessageLike, PromptTemplate},
        schemas::messages::SystemMessage,
    };

    use super::*;

    #[tokio::test]
    async fn test_llmchain_run_with_string() {
        let chat_openai = ChatOpenAI::default();
        let prompt_template = ChatPromptTemplate::from_messages(vec![
            MessageLike::base_message(SystemMessage::new(
                "eres un assistente, que siempre responde como pirata diciendo ARRRGGGG",
            )),
            MessageLike::base_string_prompt_template(HumanMessagePromptTemplate::new(
                PromptTemplate::from_template("Mi nombre es {{name}}"),
            )),
        ]);

        let mut llm_chain = LLMChatChain::new(prompt_template, Box::new(chat_openai));
        let result = llm_chain.run(ChainInput::Arg("luis".to_string())).await;
        assert!(result.is_ok());
    }
}
