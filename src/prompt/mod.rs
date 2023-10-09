mod chat;
mod prompt;
pub use chat::*;
pub use prompt::{BasePromptTemplate, PromptTemplate, StringPromptValue};

use serde_json::Value;
use std::{collections::HashMap, error::Error};

pub trait TemplateArgs: Sync + Send {
    fn to_map(&self, input_variables: &[String]) -> Result<HashMap<String, Value>, Box<dyn Error>>;
    fn clone_as_map(&self) -> HashMap<String, Value>;
}

impl TemplateArgs for String {
    fn to_map(&self, input_variables: &[String]) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        if input_variables.len() == 1 {
            let mut map = HashMap::new();
            map.insert("input".to_string(), Value::String(self.clone()));
            Ok(map)
        } else {
            Err(Box::new(std::fmt::Error))
        }
    }

    fn clone_as_map(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("input".to_string(), Value::String(self.clone()));
        map
    }
}

impl TemplateArgs for HashMap<String, Value> {
    fn to_map(&self, _: &[String]) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        Ok(self.clone())
    }
    fn clone_as_map(&self) -> HashMap<String, Value> {
        self.clone()
    }
}
