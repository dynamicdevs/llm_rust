mod chat;
mod prompt;
pub use chat::*;
pub use prompt::{BasePromptTemplate, PromptTemplate, StringPromptValue};

use serde_json::Value;
use std::{collections::HashMap, error::Error};

pub trait TemplateArgs {
    fn to_map(&self, input_variables: &[String]) -> Result<HashMap<String, Value>, Box<dyn Error>>;
}

impl TemplateArgs for String {
    fn to_map(&self, input_variables: &[String]) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        if input_variables.len() == 1 {
            let mut map = HashMap::new();
            map.insert(input_variables[0].clone(), Value::String(self.clone()));
            Ok(map)
        } else {
            Err(Box::new(std::fmt::Error))
        }
    }
}

impl TemplateArgs for HashMap<String, Value> {
    fn to_map(&self, _: &[String]) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        Ok(self.clone())
    }
}
