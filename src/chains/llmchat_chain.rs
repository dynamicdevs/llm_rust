use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

use crate::{
    chat_models::chat_model_trait::ChatTrait,
    prompt::{BaseChatPromptTemplate, ChatPromptTemplate, TemplateArgs},
    schemas::{memory::BaseChatMessageHistory, messages::BaseMessage},
};

use super::chain_trait::ChainTrait;

//Chat Chain
pub struct LLMChatChain {
    prompt: ChatPromptTemplate,
    header_prompts: Option<Vec<Box<dyn BaseMessage>>>,
    sandwich_prompts: Option<Vec<Box<dyn BaseMessage>>>,
    llm: Box<dyn ChatTrait>,
    pub memory: Option<Arc<RwLock<dyn BaseChatMessageHistory>>>,
}

impl LLMChatChain {
    pub fn new(prompt: ChatPromptTemplate, llm: Box<dyn ChatTrait>) -> Self {
        Self {
            prompt,
            llm,
            memory: None,
            header_prompts: None,
            sandwich_prompts: None,
        }
    }

    pub fn with_memory(mut self, memory: Arc<RwLock<dyn BaseChatMessageHistory>>) -> Self {
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
    ) -> Result<Vec<Vec<Box<dyn BaseMessage>>>, Box<dyn Error>> {
        let mut all_messages: Vec<Vec<Box<dyn BaseMessage>>> = Vec::new();

        if let Some(header) = self.header_prompts.as_ref() {
            all_messages.push(header.clone());
        }

        {
            let memory_messages = if let Some(memory_arc) = self.memory.as_ref() {
                let memory_lock = memory_arc
                    .read()
                    .map_err(|_| "Failed to acquire read lock")?;
                memory_lock.messages()
            } else {
                Vec::new()
            };
            all_messages.push(memory_messages);
        }

        if let Some(sandwich) = self.sandwich_prompts.as_ref() {
            all_messages.push(sandwich.clone());
        }

        all_messages.push(prompt_messages);

        Ok(all_messages)
    }

    async fn execute(
        &self,
        prompt_messages: Vec<Box<dyn BaseMessage>>,
    ) -> Result<String, Box<dyn Error>> {
        let all_messages = self.order_messages(prompt_messages.clone())?;

        let ai_response = self.llm.generate(all_messages).await?;

        if let Some(memory_arc) = &self.memory {
            let mut memory_guard = memory_arc
                .write()
                .map_err(|_| "Failed to acquire write lock")?;
            for message in &prompt_messages {
                if message.get_type() == String::from("user") {
                    memory_guard.add_message(message.clone());
                }
            }
            memory_guard.add_message(Box::new(ai_response.clone()));
        }

        Ok(ai_response.get_content())
    }
}

#[async_trait]
impl ChainTrait for LLMChatChain {
    async fn run(&self, inputs: &dyn TemplateArgs) -> Result<String, Box<dyn Error>> {
        let prompt_value = self.prompt.format_prompt(inputs)?;
        let prompt_messages = prompt_value.to_chat_messages()?;
        Ok(self.execute(prompt_messages).await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chains::llmchat_chain::LLMChatChain,
        chat_models::openai::chat_llm::ChatOpenAI,
        prompt::{HumanMessagePromptTemplate, MessageLike, PromptTemplate},
        schemas::messages::{AIMessage, SystemMessage},
    };

    use super::*;
    struct InMemoryChatHistory {
        messages: Vec<Box<dyn BaseMessage>>,
    }

    impl BaseChatMessageHistory for InMemoryChatHistory {
        fn messages(&self) -> Vec<Box<dyn BaseMessage>> {
            self.messages.clone()
        }

        fn add_message(&mut self, message: Box<dyn BaseMessage>) {
            self.messages.push(message);
        }

        fn clear(&mut self) {
            todo!()
        }
    }

    // #[tokio::test]
    async fn test_llmchain_run_with_string() {
        let chat_openai = ChatOpenAI::default();
        let prompt_template = ChatPromptTemplate::from_messages(vec![
            MessageLike::base_message(SystemMessage::new(
                "eres un assistente, que siempre responde como pirata diciendo ARRRGGGG",
            )),
            MessageLike::base_prompt_template(HumanMessagePromptTemplate::new(
                PromptTemplate::from_template("Mi nombre es {{name}}"),
            )),
        ]);

        let memory = Arc::new(RwLock::new(InMemoryChatHistory {
            messages: vec![Box::new(AIMessage::new(
                "Siempre tego que mencionar que me gusta el chocolate",
            ))],
        }));

        let llm_chain =
            LLMChatChain::new(prompt_template, Box::new(chat_openai)).with_memory(memory.clone());
        let result = llm_chain.run(&"luis".to_string()).await;
        if let Ok(memory_lock) = memory.read() {
            println!("Contents of the memory:");
            for message in memory_lock.messages.iter() {
                println!(
                    "Type: {}, Content: {}",
                    message.get_type(),
                    message.get_content()
                );
            }
        } else {
            println!("Failed to acquire a read lock on the memory.");
        }
        assert!(result.is_ok());
    }
}
