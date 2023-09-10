use async_trait::async_trait;

use crate::{
    llm::{base::BaseLLM, openai::LLMOpenAI},
    prompt::prompt::PromptTemplate,
    schemas::prompt::{BasePromptValue, PromptData},
};

use super::chain_trait::ChainTrait;

pub struct LLMChain {
    llm: Box<dyn BaseLLM>,
    prompt: PromptTemplate,
}
impl LLMChain {
    pub fn new(llm: Box<dyn BaseLLM>, prompt: PromptTemplate) -> Self {
        Self { llm, prompt }
    }
}

impl Default for LLMChain {
    fn default() -> Self {
        Self {
            llm: Box::new(LLMOpenAI::default()),
            prompt: PromptTemplate::new("{{question}}"),
        }
    }
}

#[async_trait]
impl ChainTrait<String> for LLMChain {
    async fn run(&mut self, inputs: String) -> Result<String, Box<dyn std::error::Error>> {
        self.prompt.add_values(PromptData::VecData(vec![inputs]));
        let prompt = self.prompt.render()?;
        Ok(self.llm.generate(prompt).await?)
    }
}
