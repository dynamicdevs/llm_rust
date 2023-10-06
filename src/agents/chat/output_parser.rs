use std::error::Error;

use crate::{
    agents::agent::AgentOutputParser,
    schemas::agent::{AgentAction, AgentEvent, AgentFinish},
};
use regex::Regex;
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
        log::debug!("Parsing to Agent Action: {}", text);
        let re = Regex::new(r"\{(?:[^{}]|(?R))*\}")?;
        let json_match = re.find(text);
        let parsed_json: Value = match json_match {
            Some(json_str) => serde_json::from_str(json_str.as_str())?,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("No JSON found in text: `{}`", text),
                )))
            }
        };

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
