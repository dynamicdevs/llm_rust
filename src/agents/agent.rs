use std::{error::Error, sync::Arc};

use async_trait::async_trait;

use crate::{
    prompt::TemplateArgs,
    schemas::agent::{AgentAction, AgentEvent, AgentPlan},
    tools::tool_trait::Tool,
};

#[async_trait]
pub trait Agent: Send + Sync {
    async fn plan(
        &self,
        intermediate_steps: &Vec<(AgentAction, String)>,
        inputs: &dyn TemplateArgs,
    ) -> Result<AgentPlan, Box<dyn Error>>;

    fn get_tools(&self) -> Vec<Arc<dyn Tool>>;
}

pub trait AgentOutputParser: Send + Sync {
    fn parse(&self, text: &str) -> Result<AgentEvent, Box<dyn Error>>;
    fn get_format_instructions(&self) -> &str;
}
