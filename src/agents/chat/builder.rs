use std::sync::Arc;

use crate::{
    agents::agent::AgentOutputParser, chains::llmchat_chain::LLMChatChain, tools::tool_trait::Tool,
};

use super::{
    prompt::{FORMAT_INSTRUCTIONS, PREFIX, SUFFIX, TEMPLATE_TOOL_RESPONSE},
    ConversationalAgent, ConvoOutputParser,
};

pub struct ConversationalAgentBuilder {
    llm: Option<Box<dyn crate::chat_models::chat_model_trait::ChatTrait>>,
    tools: Option<Vec<Arc<dyn Tool>>>,
    output_parser: Option<Box<dyn AgentOutputParser>>,
    prefix: Option<String>,
    suffix: Option<String>,
    template_tool_response: Option<String>,
}

impl ConversationalAgentBuilder {
    pub fn new() -> Self {
        Self {
            llm: None,
            tools: None,
            output_parser: None,
            prefix: None,
            suffix: None,
            template_tool_response: None,
        }
    }

    pub fn llm(mut self, llm: Box<dyn crate::chat_models::chat_model_trait::ChatTrait>) -> Self {
        self.llm = Some(llm);
        self
    }

    pub fn tools(mut self, tools: Vec<Arc<dyn Tool>>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn output_parser(mut self, parser: Box<dyn AgentOutputParser>) -> Self {
        self.output_parser = Some(parser);
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    pub fn suffix(mut self, suffix: &str) -> Self {
        self.suffix = Some(suffix.to_string());
        self
    }

    pub fn template_tool_response(mut self, template: &str) -> Self {
        self.template_tool_response = Some(template.to_string());
        self
    }

    pub fn build(self) -> Result<ConversationalAgent, Box<dyn std::error::Error>> {
        let llm = self.llm.ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "LLM is not provided.",
            ))
        })?;
        let tools = self.tools.ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Tools are not provided.",
            ))
        })?;
        let output_parser = self
            .output_parser
            .unwrap_or_else(|| Box::new(ConvoOutputParser::new()) as Box<dyn AgentOutputParser>);

        let prefix = self.prefix.unwrap_or_else(|| PREFIX.to_string());
        let suffix = self.suffix.unwrap_or_else(|| SUFFIX.to_string());
        let template_tool_response = self
            .template_tool_response
            .unwrap_or_else(|| TEMPLATE_TOOL_RESPONSE.to_string());

        let prompt =
            ConversationalAgent::create_prompt(&tools, &prefix, &suffix, &FORMAT_INSTRUCTIONS)?;
        let chain = Box::new(LLMChatChain::new(prompt, llm));

        Ok(ConversationalAgent {
            tools,
            chain,
            system_message: prefix,
            human_message: suffix,
            output_parser,
            template_tool_response,
        })
    }
}
