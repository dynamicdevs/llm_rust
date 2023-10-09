use crate::{
    llm::{base::BaseLLM, openai::LLMOpenAI},
    prompt::{BasePromptTemplate, PromptTemplate, TemplateArgs},
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
            prompt: PromptTemplate::from_template("{{input}}"),
        }
    }
}

#[async_trait]
impl ChainTrait for LLMChain {
    async fn run(&self, inputs: &dyn TemplateArgs) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = self.prompt.format(inputs)?;
        Ok(self.llm.generate(prompt).await?)
    }
}
