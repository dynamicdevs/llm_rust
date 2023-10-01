use crate::agents::agent::AgentOutputParser;

use super::prompt::FORMAT_INSTRUCTIONS;

pub struct ConvoOutputParser {}

impl AgentOutputParser for ConvoOutputParser {
    fn parse(&self, text: &str) -> crate::schemas::agent::AgentEvent {
        unimplemented!()
    }

    fn get_format_instructions(&self) -> &str {
        FORMAT_INSTRUCTIONS
    }
}
// def get_format_instructions(self) -> str:
//     """Returns formatting instructions for the given output parser."""
//     return FORMAT_INSTRUCTIONS
