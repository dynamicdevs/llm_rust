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
}

impl AgentOutputParser for ConvoOutputParser {
    fn parse(&self, text: &str) -> Result<AgentEvent, Box<dyn Error>> {
        let sanitized_text = text
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect::<String>();

        log::debug!("Parsing to Agent Action: {}", sanitized_text);
        let re = Regex::new(r"```json?\s*(.*?)\s*```").unwrap();
        let json_match = re.captures(&sanitized_text).and_then(|cap| cap.get(1));
        log::debug!("Finish extracting json");
        let agent_output: AgentOutput = match json_match {
            Some(json_str) => serde_json::from_str(&json_str.as_str())?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_within_backticks() {
        let parser = ConvoOutputParser::new();
        let input = r#"```json
    {
        "action": "Product_information",
        "action_input": "{\"products\":[\"Laptop Lenovo LOQ 15IRH8 15.6 Core i7-13620H 16G 512 SSD V6G\"],\"query\":\"Dame mas info\"}"
    }
    ```"#;

        match parser.parse(input) {
            Ok(AgentEvent::Action(action)) => {
                println!("Action Tool Input: {}", action.tool_input);
                assert_eq!(action.tool, "Product_information");
                // Corrected the expected value of `tool_input` for the assertion
                assert_eq!(action.tool_input, "{\"products\":[\"Laptop Lenovo LOQ 15IRH8 15.6 Core i7-13620H 16G 512 SSD V6G\"],\"query\":\"Dame mas info\"}");
            }
            Ok(_result) => {
                // Print the unexpected result for diagnostic purposes
                panic!("Expected an AgentAction but got something else.");
            }
            Err(e) => {
                // Print the error for diagnostic purposes
                println!("Error: {}", e);
                panic!("Error occurred during parsing.");
            }
        }
    }

    #[test]
    fn test_no_json_within_backticks() {
        let parser = ConvoOutputParser::new();
        let input = r"some text without any json backticks";

        match parser.parse(input) {
            Ok(AgentEvent::Finish(finish)) => {
                assert_eq!(finish.return_values, input);
            }
            _ => panic!("Expected an AgentFinish but got something else."),
        }
    }

    // You can add more tests as required
}
