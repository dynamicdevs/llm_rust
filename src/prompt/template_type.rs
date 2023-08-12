use handlebars::Handlebars;
use regex::Regex;
use std::collections::HashMap;

use crate::errors::PromptError;

pub struct Prompt {
    template: String,
    data: PromptData,
}

enum PromptData {
    VecData(Vec<String>),
    HashMapData(HashMap<String, String>),
}

impl Prompt {
    pub fn new(template: &str) -> Self {
        Prompt {
            template: template.to_string(),
            data: PromptData::VecData(Vec::new()),
        }
    }

    pub fn new_from_vec(template: &str, data: Vec<String>) -> Result<String, PromptError> {
        let prompt = Prompt {
            template: template.to_string(),
            data: PromptData::VecData(data),
        };

        prompt.render()
    }

    pub fn new_from_hashmap(
        template: &str,
        data: HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let prompt = Prompt {
            template: template.to_string(),
            data: PromptData::HashMapData(data),
        };

        prompt.render()
    }

    pub fn render(&self) -> Result<String, PromptError> {
        let re = Regex::new(r"\{\{([^\{\}]+)\}\}").unwrap();
        let captures: Vec<String> = re
            .captures_iter(&self.template)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .collect();

        let mut map = HashMap::new();
        match &self.data {
            PromptData::VecData(data_vec) => {
                for (index, placeholder) in captures.iter().enumerate() {
                    if let Some(value) = data_vec.get(index) {
                        map.insert(placeholder.clone(), value.clone());
                    }
                }
            }
            PromptData::HashMapData(data_map) => {
                map.extend(data_map.clone());
            }
        }

        let handlebars = Handlebars::new();
        handlebars
            .render_template(&self.template, &map)
            .map_err(|e| PromptError::RenderError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_from_vec() {
        let template = "Hello {{name}} and {{test}}!";
        let vec_data = vec!["Alice".to_string(), "Bob".to_string()];
        let result = Prompt::new_from_vec(template, vec_data);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "Hello Alice and Bob!");
    }

    #[test]
    fn test_render_from_hashmap() {
        let template = "Hello {{first}} and {{second}}!";
        let mut map_data = HashMap::new();
        map_data.insert("first".to_string(), "Charlie".to_string());
        map_data.insert("second".to_string(), "David".to_string());
        let result = Prompt::new_from_hashmap(template, map_data);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "Hello Charlie and David!");
    }
}
