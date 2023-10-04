use std::error::Error;

use async_trait::async_trait;

use crate::{
    prompt::TemplateArgs,
    schemas::agent::{AgentAction, AgentEvent},
};

#[async_trait]
pub trait Agent: Send + Sync {
    async fn plan(
        &self,
        intermediate_steps: &Vec<(AgentAction, String)>,
        inputs: &dyn TemplateArgs,
    ) -> Result<AgentEvent, Box<dyn Error>>;
}

pub trait AgentOutputParser: Send + Sync {
    fn parse(&self, text: &str) -> Result<AgentEvent, Box<dyn Error>>;
    fn get_format_instructions(&self) -> &str;
}
