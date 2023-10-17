use std::error::Error;

use crate::{
    agents::agent::AgentOutputParser,
    schemas::agent::{AgentAction, AgentEvent, AgentFinish},
};
use regex::Regex;
use serde::Deserialize;

use super::prompt::FORMAT_INSTRUCTIONS;

#[derive(Debug, Deserialize)]
struct AgentOutput {
    action: String,
    action_input: String,
}

pub struct ConvoOutputParser {}
impl ConvoOutputParser {
    pub fn new() -> Self {
        Self {}
    }

    fn clean_json_match(&self, matched: &str) -> String {
        matched.replace("\\\"", "\"")
    }
}

impl AgentOutputParser for ConvoOutputParser {
    fn parse(&self, text: &str) -> Result<AgentEvent, Box<dyn Error>> {
        let sanitized_text = text
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect::<String>();

        log::debug!("Parsing to Agent Action: {}", sanitized_text);
        let re = Regex::new(r"^\{.*\}$")?;
        let json_match = re.find(&sanitized_text);
        log::debug!("Finish extracting json");
        let agent_output: AgentOutput = match json_match {
            Some(json_str) => {
                let cleaned_str = self.clean_json_match(json_str.as_str());
                log::debug!("Cleaned Json:{:?}", cleaned_str);
                serde_json::from_str(&cleaned_str)?
            }
            None => {
                log::debug!("No JSON found in text: {}", sanitized_text);
                return Ok(AgentEvent::Finish(AgentFinish {
                    return_values: sanitized_text,
                }));
            }
        };

        if &agent_output.action == "Final Answer" {
            Ok(AgentEvent::Finish(AgentFinish {
                return_values: agent_output.action_input,
            }))
        } else {
            Ok(AgentEvent::Action(AgentAction {
                tool: agent_output.action,
                tool_input: agent_output.action_input,
                log: sanitized_text,
            }))
        }
    }

    fn get_format_instructions(&self) -> &str {
        FORMAT_INSTRUCTIONS
    }
}
