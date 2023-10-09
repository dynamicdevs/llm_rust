use std::{error::Error, sync::Arc};

use crate::{
    chains::{chain_trait::ChainTrait, llmchat_chain::LLMChatChain},
    prompt::{
        ChatPromptTemplate, HumanMessagePromptTemplate, MessageLike, MessagesPlaceholder,
        PromptTemplate, TemplateArgs,
    },
    schemas::{
        agent::{AgentAction, AgentEvent},
        chain::ChainResponse,
        messages::{AIMessage, BaseMessage, HumanMessage, SystemMessage},
    },
    tools::tool_trait::Tool,
};
use async_trait::async_trait;
use handlebars::Handlebars;
use serde_json::json;

use self::prompt::{FORMAT_INSTRUCTIONS, PREFIX, SUFFIX, TEMPLATE_TOOL_RESPONSE};

use super::agent::{Agent, AgentOutputParser};

pub mod builder;
pub use builder::ConversationalAgentBuilder;
pub mod output_parser;
pub use output_parser::ConvoOutputParser;
mod prompt;

pub struct ConversationalAgent {
    tools: Vec<Arc<dyn Tool>>,
    chain: Box<dyn ChainTrait>,
    system_message: String,
    human_message: String,
    output_parser: Box<dyn AgentOutputParser>,
    template_tool_response: String,
}

impl ConversationalAgent {
    fn create_prompt(
        tools: &Vec<Arc<dyn Tool>>,
        system_message: &str,
        human_message: &str,
        format_instruction: &str,
    ) -> Result<ChatPromptTemplate, Box<dyn Error>> {
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

        let handlebars = Handlebars::new();
        let format_instruction = handlebars.render_template(
            human_message,
            &json!({
                "format_instructions":format_instruction,
                "tools":"{{tools}}",
                "input":"{{input}}"
            }),
        )?;

        let final_prompt = handlebars.render_template(
            &format_instruction,
            &json!({
                    "tool_names":tool_names,
                    "tools":tool_string,
                    "input":"{{input}}"
            }),
        )?;
        let prompt = html_escape::decode_html_entities(&final_prompt).to_string();
        log::debug!("Prompt:{}", prompt);

        let prompt = ChatPromptTemplate::from_messages(vec![
            MessageLike::base_message(SystemMessage::new(system_message)),
            MessageLike::base_prompt_template(MessagesPlaceholder::new("chat_history")),
            MessageLike::base_prompt_template(HumanMessagePromptTemplate::new(
                PromptTemplate::from_template(&prompt),
            )),
            MessageLike::base_prompt_template(MessagesPlaceholder::new("agent_scratchpad")),
        ]);

        Ok(prompt)
    }

    fn construct_scratchpad(
        &self,
        intermediate_steps: &Vec<(AgentAction, String)>,
    ) -> Result<Vec<Box<dyn BaseMessage>>, Box<dyn Error>> {
        log::debug!("Building scratchpad");
        let mut thoughts: Vec<Box<dyn BaseMessage>> = Vec::new();

        for (action, observation) in intermediate_steps.into_iter() {
            log::debug!("Action: {:?}:{}", action, observation);
            thoughts.push(Box::new(AIMessage::new(&action.log)) as Box<dyn BaseMessage>);
            let handlebars = Handlebars::new();
            let tool_response = handlebars.render_template(
                self.template_tool_response.as_str(),
                &json!({ "observation": observation }),
            )?;
            thoughts.push(Box::new(HumanMessage::new(&tool_response)));
        }

        Ok(thoughts)
    }

    pub fn from_llm_and_tools(
        llm: Box<dyn crate::chat_models::chat_model_trait::ChatTrait>,
        tools: Vec<Arc<dyn Tool>>,
        output_parser: Box<dyn AgentOutputParser>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let prompt =
            ConversationalAgent::create_prompt(&tools, &PREFIX, &SUFFIX, &FORMAT_INSTRUCTIONS)?;
        let chain = Box::new(LLMChatChain::new(prompt, llm));
        Ok(Self {
            tools,
            chain,
            system_message: PREFIX.to_string(),
            human_message: SUFFIX.to_string(),
            output_parser,
            template_tool_response: TEMPLATE_TOOL_RESPONSE.to_string(),
        })
    }
}
#[async_trait]
impl Agent for ConversationalAgent {
    async fn plan(
        &self,
        intermediate_steps: &Vec<(AgentAction, String)>,
        inputs: &dyn TemplateArgs,
    ) -> Result<AgentEvent, Box<dyn Error>> {
        log::debug!("Planning");
        let scratchpad = self.construct_scratchpad(&intermediate_steps)?;
        let mut inputs = inputs.clone_as_map();
        inputs.insert("agent_scratchpad".to_string(), json!(scratchpad)); // Assuming scratchpad is a Stringhapad

        log::debug!("Running chain");
        let output = self.chain.run(&inputs).await?;
        match output {
            ChainResponse::Text(output) => {
                log::debug!("Parsing output:{}", output);
                let parsed_output = self.output_parser.parse(&output)?;
                log::debug!("Parsed output");
                Ok(parsed_output)
            }
            ChainResponse::Stream(_) => {
                unimplemented!()
            }
        }
    }

    fn get_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use async_trait::async_trait;

    use crate::{
        agents::{
            chat::{output_parser::ConvoOutputParser, ConversationalAgent},
            executor::AgentExecutor,
        },
        chains::chain_trait::ChainTrait,
        schemas::chain::ChainResponse,
        tools::tool_trait::Tool,
    };

    #[derive(Debug, Clone)]
    pub struct MockPeruPresidentTool;
    #[async_trait]
    impl Tool for MockPeruPresidentTool {
        fn name(&self) -> String {
            "Get the current president".to_string()
        }

        fn description(&self) -> String {
            "A wrapper around Google Search. Useful for when you need to answer questions about current events. Input should be a search query.".to_string()
        }

        async fn call(&self, _input: &str) -> Result<String, Box<dyn Error>> {
            Ok("Luis Fernando is the president of Peru. tiene 30 anos".to_string())
        }
    }
    #[derive(Debug, Clone)]
    pub struct CalcTool;
    #[async_trait]
    impl Tool for CalcTool {
        fn name(&self) -> String {
            "Calculator".to_string()
        }

        fn description(&self) -> String {
            "Use this tool if you want to calculate ages".to_string()
        }

        async fn call(&self, _input: &str) -> Result<String, Box<dyn Error>> {
            Ok("50".to_string())
        }
    }

    #[tokio::test]
    async fn test_agent_run_with_string() {
        let agent = ConversationalAgent::from_llm_and_tools(
            Box::new(crate::chat_models::openai::chat_llm::ChatOpenAI::default()),
            vec![Arc::new(MockPeruPresidentTool), Arc::new(CalcTool)],
            Box::new(ConvoOutputParser::new()),
        );

        let exec = AgentExecutor::from_agent(Box::new(agent.unwrap()));

        let result = exec
            .run(&String::from(
                "Quien es el presidente de peru y cual es su edad multiplicada por 3",
            ))
            .await
            .map_err(|e| println!("{}", e));
        match result {
            Ok(ChainResponse::Text(text)) => {
                println!("{}", text);
            }
            Ok(ChainResponse::Stream(_)) => {
                println!("Stream");
            }
            Err(_) => {}
        }
    }
}
