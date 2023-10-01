use std::{collections::HashMap, error::Error};

use async_trait::async_trait;

use crate::{
    chat_models::chat_model_trait::ChatTrait,
    schemas::agent::{AgentAction, AgentEvent, AgentFinish},
    tools::tool_trait::Tool,
};

#[async_trait]
pub trait Agent {
    fn return_values(&self) -> Vec<String> {
        vec![String::from("output")]
    }

    async fn plan(&self, intermediate_steps: &Vec<(AgentAction, String)>) -> AgentEvent;

    async fn return_stopped_response(
        &self,
        early_stopping_method: &str,
        intermediate_steps: Vec<(AgentAction, String)>,
    ) -> Result<AgentFinish, Box<dyn Error>> {
        if early_stopping_method == "force" {
            return Ok(AgentFinish {
                return_values: "Agent stopped due to iteration limit or time limit.".to_string(),
            });
        } else {
            return Err(format!(
                "Got unsupported early_stopping_method `{}`",
                early_stopping_method
            )
            .into());
        }
    }

    fn get_tools(&self) -> Vec<Box<dyn Tool>>;

    fn from_llm_and_tools(
        llm: Box<dyn ChatTrait>,
        tools: Vec<Box<dyn Tool>>,
    ) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        Err("Not implemented".into())
    }

    fn observation_prefix(&self) -> String;

    fn llm_prefix(&self) -> String;

    fn create_promopt(&self) -> String;

    fn construct_scratchpad(
        &self,
        intermediate_steps: Vec<(AgentAction, String)>,
    ) -> Result<String, Box<dyn Error>> {
        let mut thoughts = String::new();
        for (_action, observation) in intermediate_steps {
            thoughts += &format!(
                "\n{}{}\n{}",
                self.observation_prefix(),
                observation,
                self.llm_prefix()
            );
        }
        Ok(thoughts)
    }

    fn get_full_inputs(
        &self,
        intermediate_steps: Vec<(AgentAction, String)>,
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let thoughts = self.construct_scratchpad(intermediate_steps)?;
        let mut full_inputs = HashMap::new();
        full_inputs.insert("agent_scratchpad".to_string(), thoughts);
        Ok(full_inputs)
    }
}

pub trait AgentOutputParser {
    fn parse(&self, text: &str) -> AgentEvent;
    fn get_format_instructions(&self) -> &str;
}
