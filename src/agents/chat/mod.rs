use std::error::Error;

use crate::{
    chains::{chain_trait::ChainTrait, llmchat_chain::LLMChatChain},
    prompt::prompt::{PromptTemplate, PromptTemplates},
    schemas::{
        messages::{BaseMessage, HumanMessage, SystemMessage},
        prompt::BasePromptValue,
    },
    tools::tool_trait::Tool,
};
use handlebars::Handlebars;
use serde_json::json;

use super::agent::{Agent, AgentOutputParser};

mod output_parser;
mod prompt;

pub struct ConversationalAgent {
    llm: Box<dyn ChainTrait<String>>,
    tools: Vec<Box<dyn Tool>>,
    system_message: String,
    human_message: String,
    output_parser: Box<dyn AgentOutputParser>,
}

impl ConversationalAgent {
    fn create_prompt(
        &self,
        tools: Vec<Box<dyn Tool>>,
        system_message: &str,
        human_message: &str,
    ) -> Result<Box<dyn BasePromptValue>, Box<dyn Error>> {
        let tool_string = tools
            .iter()
            .map(|tool| format!("> {}: {}", tool.name(), tool.description()))
            .collect::<Vec<_>>()
            .join("\n");
        let tool_names = tools
            .iter()
            .map(|tool| tool.name())
            .collect::<Vec<_>>()
            .join(", ");

        let mut handlebars = Handlebars::new();
        let format_instruction = handlebars.render_template(
            human_message,
            &json!({
                "format_instructions":self.output_parser.get_format_instructions(),
                "tools":"",
                "input":""
            }),
        )?;

        let final_prompt = handlebars.render_template(
            &format_instruction,
            &json!({
                    "format_instructions":"",
                    "tools":tool_string,
                    "input":""
            }),
        )?;

        let messages: Vec<Box<dyn BaseMessage>> =
            vec![Box::new(SystemMessage::new(system_message)) as Box<dyn BaseMessage>];

        let prompt: PromptTemplates = PromptTemplates::new(vec![
            Box::new(SystemMessage::new(system_message)) as Box<dyn BaseMessage>, //TODO:agreagar el
                                                                                  //el template para base message
        ]);

        unimplemented!()
    }
}

// impl Agent for ConversationalAgent {
//     fn from_llm_and_tools(
//         llm: Box<dyn crate::chat_models::chat_model_trait::ChatTrait>,
//         tools: Vec<Box<dyn Tool>>,
//     ) -> Result<Self, Box<dyn std::error::Error>>
//     where
//         Self: Sized,
//     {
//         unimplemented!()
//     }
//
//
// }
