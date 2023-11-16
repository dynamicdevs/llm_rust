use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{
    chains::chain_trait::ChainTrait,
    prompt::TemplateArgs,
    schemas::{
        agent::{AgentAction, AgentEvent, AgentPlan},
        chain::ChainResponse,
        memory::BaseChatMessageHistory,
        messages::{AIMessage, BaseMessage, HumanMessage},
    },
    tools::tool_trait::Tool,
};

use super::agent::Agent;

pub struct AgentExecutor {
    agent: Box<dyn Agent>,
    max_iterations: Option<i32>,
    pub memory: Option<Arc<RwLock<dyn BaseChatMessageHistory>>>,
}

impl AgentExecutor {
    fn get_name_to_tools(&self) -> HashMap<String, Arc<dyn Tool>> {
        let mut name_to_tool = HashMap::new();
        for tool in self.agent.get_tools().iter() {
            log::debug!("Loading Tool:{}", tool.name());
            name_to_tool.insert(tool.name(), tool.clone());
        }
        name_to_tool
    }

    pub fn from_agent(agent: Box<dyn Agent>) -> Self {
        Self {
            agent,
            max_iterations: Some(10),
            memory: None,
        }
    }
    pub fn with_memory(mut self, memory: Arc<RwLock<dyn BaseChatMessageHistory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn with_max_iterations(mut self, max_iterations: i32) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }
}

#[async_trait]
impl ChainTrait for AgentExecutor {
    async fn run(&self, input: &dyn TemplateArgs) -> Result<ChainResponse, Box<dyn Error>> {
        let name_to_tools = self.get_name_to_tools();

        let mut steps: Vec<(AgentAction, String)> = Vec::new();

        log::debug!("Starting agent");
        let mut max_iterations = self.max_iterations;

        let mut input_map = input.clone_as_map();

        if let Some(memory_arc) = &self.memory {
            let memory_guard = memory_arc
                .read()
                .map_err(|_| "Failed to acquire read lock")?;
            let message_history = memory_guard;

            log::debug!("Messaage History");
            for message in message_history.messages().iter() {
                log::debug!("{}", message.get_content());
            }
            input_map.insert(
                "chat_history".to_string(),
                serde_json::json!(message_history.messages()),
            );
        } else {
            let empty_history = vec![] as Vec<Box<dyn BaseMessage>>;
            input_map.insert("chat_history".to_string(), serde_json::json!(empty_history));
        }

        loop {
            let agent_event = self.agent.plan(&steps, &input_map).await?;
            match agent_event {
                AgentPlan::Text(event) => {
                    match event {
                        AgentEvent::Action(action) => {
                            println!("Action: {:?}", action.tool_input);
                            log::debug!("Action: {:?}", action.tool_input);
                            let tool = name_to_tools.get(&action.tool).ok_or("Tool not found")?; //No se si
                                                                                                 //lanzar error o poner este mensage evaluar
                            let observarion = tool.call(&action.tool_input).await?;
                            steps.push((action, observarion));
                        }
                        AgentEvent::Finish(finish) => {
                            println!("Finish: {:?}", finish.return_values);
                            log::debug!("AgentEvent::Finish branch entered");

                            if let Some(memory_arc) = &self.memory {
                                log::debug!("Attempting to add to memory");
                                let mut memory_guard = memory_arc
                                    .write()
                                    .map_err(|_| "Failed to acquire write lock")?;
                                log::debug!("Successfully acquired write lock");

                                let inputs = input.clone_as_map();
                                let human_str = inputs
                                    .get("input")
                                    .ok_or("Human not found")?
                                    .as_str()
                                    .ok_or("Human not found")?
                                    .to_string();
                                log::debug!("Adding Human message: {}", human_str.to_string());
                                memory_guard.add_message(Box::new(HumanMessage::new(
                                    &human_str.to_string(),
                                )));
                                log::debug!("Successfully added Human message to memory");

                                memory_guard
                                    .add_message(Box::new(AIMessage::new(&finish.return_values)));
                                log::debug!("Successfully added AI message to memory");
                            }

                            return Ok(ChainResponse::Text(finish.return_values));
                        }
                    }
                }
                AgentPlan::Stream(mut internal_stream) => {
                    let (tx, rx) = mpsc::channel(100);

                    // Clone necessary data
                    let memory_arc_clone = self.memory.clone();

                    let inputs = input.clone_as_map();
                    let human_str = inputs
                        .get("input")
                        .ok_or("Human not found")?
                        .as_str()
                        .ok_or("Human not found")?
                        .to_string();
                    // Spawn a new asynchronous task to handle stream
                    tokio::spawn(async move {
                        let mut concatenated_stream_content = String::new();

                        while let Some(event) = internal_stream.recv().await {
                            match &event {
                                Ok(message) => {
                                    concatenated_stream_content.push_str(&message);

                                    // Stop the stream when finish_reason is not null
                                }
                                _ => {}
                            }

                            if let Err(_) = tx.send(event).await {
                                eprintln!("Failed to send the event to the channel");
                                break;
                            }
                        }

                        // Save to memory
                        save_to_memory(&memory_arc_clone, &human_str, &concatenated_stream_content);
                    });

                    return Ok(ChainResponse::Stream(rx));
                }
            }

            if let Some(max_iterations) = max_iterations.as_mut() {
                *max_iterations -= 1;
                if *max_iterations == 0 {
                    return Err("Max iterations reached".into());
                }
            }
        }
    }
}

fn save_to_memory(
    memory_arc_clone: &Option<Arc<RwLock<dyn BaseChatMessageHistory>>>,
    human_message: &str,
    concatenated_stream_content: &String,
) {
    if let Some(memory_arc) = memory_arc_clone {
        if let Ok(mut memory_guard) = memory_arc.write() {
            memory_guard.add_message(Box::new(HumanMessage::new(human_message)));
            memory_guard.add_message(Box::new(AIMessage::new(concatenated_stream_content)));
        } else {
            eprintln!("Failed to acquire write lock for memory");
        }
    }
}
