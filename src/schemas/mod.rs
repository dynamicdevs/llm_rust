use serde::Deserialize;

pub mod agent;
pub mod chain;
pub mod llm;
pub mod memory;
pub mod messages;
pub mod prompt;

//delete this when refactor
#[derive(Debug, Deserialize)]
pub struct StreamData {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
}
