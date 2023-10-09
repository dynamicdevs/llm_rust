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
        // Sanitize the input to replace control characters with spaces
        let sanitized_text = text
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect::<String>();

        log::debug!("Parsing to Agent Action: {}", sanitized_text);
        let re = Regex::new(r"\{(?:[^{}]|(?R))*\}")?;
        let json_match = re.find(&sanitized_text);
        let parsed_json: Value = match json_match {
            Some(json_str) => serde_json::from_str(json_str.as_str())?,
            None => {
                //If the model dont produce a json it will return it as final answer
                log::debug!("No JSON found in text: {}", sanitized_text);
                return Ok(AgentEvent::Finish(AgentFinish {
                    return_values: sanitized_text,
                }));
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
                    log: sanitized_text,
                }))
            }
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not parse LLM output: `{}`", sanitized_text),
            )))
        }
    }

    fn get_format_instructions(&self) -> &str {
        FORMAT_INSTRUCTIONS
    }
}
