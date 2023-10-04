use serde_json::Value;
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

use crate::{
    chains::chain_trait::ChainTrait,
    prompt::TemplateArgs,
    schemas::{
        agent::{AgentAction, AgentEvent},
        memory::BaseChatMessageHistory,
        messages::{AIMessage, BaseMessage, HumanMessage},
    },
    tools::tool_trait::Tool,
};

use super::agent::Agent;

pub struct AgentExecutor {
    agent: Box<dyn Agent>,
    max_iterations: Option<i32>,
    tools: Vec<Box<dyn Tool>>,
    pub memory: Option<Arc<RwLock<dyn BaseChatMessageHistory>>>,
}

impl AgentExecutor {
    fn get_name_to_tools(&self) -> HashMap<String, Box<dyn Tool>> {
        let mut name_to_tool = HashMap::new();
        for tool in self.tools.iter() {
            log::debug!("Loading Tool:{}", tool.name());
            name_to_tool.insert(tool.name(), tool.clone_box());
        }
        name_to_tool
    }

    pub fn from_agent_and_tools(agent: Box<dyn Agent>, tools: Vec<Box<dyn Tool>>) -> Self {
        Self {
            tools,
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
    async fn run(&self, input: &dyn TemplateArgs) -> Result<String, Box<dyn Error>> {
        let name_to_tools = self.get_name_to_tools();

        let mut steps: Vec<(AgentAction, String)> = Vec::new();

        log::debug!("Starting agent");
        let mut max_iterations = self.max_iterations;
        //if memory exist i want to get the messages of the mory an append to input chat_history
        //with the vec of those messages
        ////add code here

        let mut input_map = input.clone_as_map();

        if let Some(memory_arc) = &self.memory {
            let memory_guard = memory_arc
                .read()
                .map_err(|_| "Failed to acquire read lock")?;
            let message_history = memory_guard;
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
                AgentEvent::Action(action) => {
                    log::debug!("Action: {:?}", action.tool_input);
                    let tool = name_to_tools.get(&action.tool).ok_or("Tool not found")?; //No se si
                                                                                         //lanzar error o poner este mensage evaluar
                    let observarion = tool.call(&action.tool_input).await?;
                    steps.push((action, observarion));
                }
                AgentEvent::Finish(finish) => {
                    log::debug!("Finis: {:?}", finish);
                    if let Some(memory_arc) = &self.memory {
                        log::debug!("Adding to memory");
                        let mut memory_guard = memory_arc
                            .write()
                            .map_err(|_| "Failed to acquire write lock")?;
                        let human_str = input.clone_as_map();
                        let human_str = human_str.get("input").ok_or("Human not found")?;
                        log::debug!("Human: {:?}", human_str);
                        memory_guard
                            .add_message(Box::new(HumanMessage::new(&human_str.to_string())));
                        memory_guard.add_message(Box::new(AIMessage::new(&finish.return_values)));
                    }
                    return Ok(finish.return_values);
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
