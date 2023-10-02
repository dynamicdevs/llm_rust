use serde_json::Value;
use std::{collections::HashMap, error::Error};

use async_trait::async_trait;

use crate::prompt::TemplateArgs;

pub enum ChainInput {
    Arg(String),
    Args(HashMap<String, Value>),
}

impl TemplateArgs for ChainInput {
    fn to_map(&self, input_variables: &[String]) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        match self {
            ChainInput::Arg(s) => {
                if input_variables.len() == 1 {
                    let mut map = HashMap::new();
                    map.insert(input_variables[0].clone(), Value::String(s.clone()));
                    Ok(map)
                } else {
                    Err(Box::new(std::fmt::Error))
                }
            }
            ChainInput::Args(m) => Ok(m.clone()),
        }
    }
}

#[async_trait]
pub trait ChainTrait: Send + Sync {
    async fn run(&mut self, input: ChainInput) -> Result<String, Box<dyn Error>>;
}
