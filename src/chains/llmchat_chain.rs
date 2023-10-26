use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use futures::StreamExt;
use reqwest_eventsource::Event;
use tokio::sync::mpsc;

use crate::{
    chat_models::chat_model_trait::ChatTrait,
    prompt::{BaseChatPromptTemplate, ChatPromptTemplate, TemplateArgs},
    schemas::{
        chain::ChainResponse,
        llm::LlmResponse,
        memory::BaseChatMessageHistory,
        messages::{AIMessage, BaseMessage},
        StreamData,
    },
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
    ) -> Result<ChainResponse, Box<dyn Error>> {
        let all_messages = self.order_messages(prompt_messages.clone())?;

        let response = self.llm.generate(all_messages).await?;
        match response {
            LlmResponse::Text(response) => {
                if let Some(memory_arc) = &self.memory {
                    let mut memory_guard = memory_arc
                        .write()
                        .map_err(|_| "Failed to acquire write lock")?;
                    for message in &prompt_messages {
                        if message.get_type() == String::from("user") {
                            memory_guard.add_message(message.clone());
                        }
                    }
                    memory_guard.add_message(Box::new(AIMessage::new(&response)));
                }

                return Ok(ChainResponse::Text(response));
            }

            LlmResponse::Stream(es) => {
                let (tx, rx) = mpsc::channel::<Result<String, reqwest_eventsource::Error>>(100);

                // Clone needed data
                let memory_arc_clone = self.memory.clone();
                let prompt_messages_clone = prompt_messages.clone();

                tokio::spawn(async move {
                    let mut concatenated_stream_content = String::new();
                    let mut internal_es = es;

                    while let Some(event) = internal_es.next().await {
                        match event {
                            Ok(Event::Message(message)) => {
                                // Deserialize the JSON data
                                if let Ok(data) = serde_json::from_str::<StreamData>(&message.data)
                                {
                                    // Only concatenate delta["content"]
                                    if let Some(delta) =
                                        data.choices.get(0).and_then(|choice| choice.delta.as_ref())
                                    {
                                        if let Some(content) = &delta.content {
                                            concatenated_stream_content.push_str(content);
                                            // Send just the delta.content through the tx channel
                                            if let Err(_) = tx.send(Ok(content.clone())).await {
                                                eprintln!(
                                                    "Failed to send the content to the channel"
                                                );
                                                break;
                                            }
                                        }
                                    }

                                    // Stop the stream when finish_reason is not null
                                    if data
                                        .choices
                                        .get(0)
                                        .and_then(|choice| choice.finish_reason.as_ref())
                                        .is_some()
                                    {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error while processing the stream: {:?}", e);
                                // Send the error through the tx channel
                                if let Err(_) = tx.send(Err(e)).await {
                                    eprintln!("Failed to send the error to the channel");
                                    break;
                                }
                            }
                            // For other event types, you might want to decide how to handle them
                            _ => {}
                        }
                    }

                    // Save to memory
                    save_to_memory(
                        &memory_arc_clone,
                        &prompt_messages_clone,
                        &concatenated_stream_content,
                    );
                });

                Ok(ChainResponse::Stream(rx))
            }
        }
    }
}

fn save_to_memory(
    memory_arc_clone: &Option<Arc<RwLock<dyn BaseChatMessageHistory>>>,
    prompt_messages_clone: &Vec<Box<dyn BaseMessage>>,
    concatenated_stream_content: &String,
) {
    if let Some(memory_arc) = memory_arc_clone {
        if let Ok(mut memory_guard) = memory_arc.write() {
            for message in prompt_messages_clone {
                if message.get_type() == String::from("user") {
                    memory_guard.add_message(message.clone());
                }
            }
            memory_guard.add_message(Box::new(AIMessage::new(concatenated_stream_content)));
        } else {
            eprintln!("Failed to acquire write lock for memory");
        }
    }
}

#[async_trait]
impl ChainTrait for LLMChatChain {
    async fn run(&self, inputs: &dyn TemplateArgs) -> Result<ChainResponse, Box<dyn Error>> {
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

    #[tokio::test]
    async fn test_llmchain_run_with_string() {
        let chat_openai = ChatOpenAI::default().with_stream();
        let prompt_template = ChatPromptTemplate::from_messages(vec![
            MessageLike::base_message(SystemMessage::new(
                "eres un assistente, que siempre responde como pirata diciendo ARRRGGGG",
            )),
            MessageLike::base_prompt_template(HumanMessagePromptTemplate::new(
                PromptTemplate::from_template("Mi nombre es {{input}}"),
            )),
        ]);

        let memory = Arc::new(RwLock::new(InMemoryChatHistory {
            messages: vec![Box::new(AIMessage::new(
                "Siempre tengo que mencionar que me gusta el chocolate",
            ))],
        }));

        let llm_chain =
            LLMChatChain::new(prompt_template, Box::new(chat_openai)).with_memory(memory.clone());
        let result = llm_chain.run(&"luis".to_string()).await;

        match result {
            Ok(ChainResponse::Text(text)) => {
                println!("Returned text: {}", text);
            }
            Ok(ChainResponse::Stream(mut stream)) => {
                println!("Returned stream:");
                while let Some(event_result) = stream.recv().await {
                    match event_result {
                        Ok(message) => {
                            println!("Streamed message: {:#?}", message);
                        }
                        Err(err) => {
                            println!("Error in stream: {}", err);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error returned: {:?}", e);
            }
        }

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
        };
    }
}
