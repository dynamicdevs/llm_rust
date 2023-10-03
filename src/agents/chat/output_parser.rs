use std::error::Error;

use crate::{
    agents::agent::AgentOutputParser,
    schemas::agent::{AgentAction, AgentEvent, AgentFinish},
};
use serde_json::Value;

use super::prompt::FORMAT_INSTRUCTIONS;

pub struct ConvoOutputParser {}
impl ConvoOutputParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl AgentOutputParser for ConvoOutputParser {
    fn parse(&self, text: &str) -> Result<AgentEvent, Box<dyn Error>> {
        let parsed_json: Value = serde_json::from_str(text)?;

        if let (Some(action), Some(action_input)) = (
            parsed_json.get("action").and_then(|a| a.as_str()),
            parsed_json.get("action_input").and_then(|a| a.as_str()),
        ) {
            if action == "Final Answer" {
                Ok(AgentEvent::Finish(AgentFinish {
                    return_values: action_input.to_string(),
                }))
            } else {
                Ok(AgentEvent::Action(AgentAction {
                    tool: action.to_string(),
                    tool_input: action_input.to_string(),
                    log: text.to_string(),
                }))
            }
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not parse LLM output: `{}`", text),
            )))
        }
    }

    fn get_format_instructions(&self) -> &str {
        FORMAT_INSTRUCTIONS
    }
}
