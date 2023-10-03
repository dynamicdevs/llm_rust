use std::{collections::HashMap, error::Error};

use crate::{
    prompt::TemplateArgs,
    schemas::agent::{AgentAction, AgentEvent},
    tools::tool_trait::Tool,
};

use super::agent::Agent;

pub struct AgentExecutor {
    agent: Box<dyn Agent>,
    max_iterations: Option<i32>,
    tools: Vec<Box<dyn Tool>>,
}

impl AgentExecutor {
    pub fn from_agent_and_tools(agent: Box<dyn Agent>, tools: Vec<Box<dyn Tool>>) -> Self {
        Self {
            tools,
            agent,
            max_iterations: Some(10),
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: Option<i32>) -> Self {
        self.max_iterations = max_iterations;
        self
    }
}

impl AgentExecutor {
    pub async fn run(&self, input: &dyn TemplateArgs) -> Result<String, Box<dyn Error>> {
        let name_to_tools = self.get_name_to_tools();

        let mut steps: Vec<(AgentAction, String)> = Vec::new();

        let mut max_iterations = self.max_iterations;
        loop {
            let agent_event = self.agent.plan(&steps, input).await?;
            match agent_event {
                AgentEvent::Action(action) => {
                    let tool = name_to_tools.get(&action.tool).ok_or("Tool not found")?; //No se si
                                                                                         //lanzar error o poner este mensage evaluar
                    let observarion = tool.call(&action.tool_input)?;
                    steps.push((action, observarion));
                }
                AgentEvent::Finish(finish) => return Ok(finish.return_values),
            }

            if let Some(max_iterations) = max_iterations.as_mut() {
                *max_iterations -= 1;
                if *max_iterations == 0 {
                    return Err("Max iterations reached".into());
                }
            }
        }
    }

    fn get_name_to_tools(&self) -> HashMap<String, Box<dyn Tool>> {
        let mut name_to_tool = HashMap::new();
        for tool in self.tools.iter() {
            name_to_tool.insert(tool.name(), tool.clone_box());
        }
        name_to_tool
    }
}
