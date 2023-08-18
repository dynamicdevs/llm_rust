use handlebars::Handlebars;
use regex::Regex;
use std::collections::HashMap;

use crate::{
    errors::PromptError,
    schemas::messages::{AIMessage, BaseMessage, HumanMessage, SystemMessage},
};

pub struct PromptTemplate {
    template: String,
    data: PromptData,
}

enum PromptData {
    VecData(Vec<String>),
    HashMapData(HashMap<String, String>),
}

impl PromptTemplate {
    pub fn new(template: &str) -> Self {
        PromptTemplate {
            template: template.to_string(),
            data: PromptData::VecData(Vec::new()),
        }
    }

    pub fn new_from_vec(template: &str, data: Vec<String>) -> Result<String, PromptError> {
        let prompt = PromptTemplate {
            template: template.to_string(),
            data: PromptData::VecData(data),
        };

        prompt.render()
    }

    pub fn new_from_hashmap(
        template: &str,
        data: HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let prompt = PromptTemplate {
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

pub struct PromptTemplates {
    templates: Vec<Box<dyn BaseMessage>>,
}

impl PromptTemplates {
    pub fn from_messages(templates: Vec<Box<dyn BaseMessage>>) -> Self {
        PromptTemplates { templates }
    }

    pub fn from_str(message: &str) -> Self {
        PromptTemplates::from_messages(vec![Box::new(HumanMessage {
            content: message.to_string(),
        })])
    }

    pub fn render_from_vec(
        &self,
        data: Vec<String>,
    ) -> Result<Vec<Box<dyn BaseMessage>>, PromptError> {
        let mut all_placeholders = Vec::new();
        for template in &self.templates {
            let re = Regex::new(r"\{\{([^\{\}]+)\}\}").unwrap();
            let captures: Vec<String> = re
                .captures_iter(&template.get_content())
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
                .collect();
            all_placeholders.extend(captures);
        }

        let mut map = HashMap::new();
        for (placeholder, value) in all_placeholders.iter().zip(data.iter()) {
            map.insert(placeholder.clone(), value.clone());
        }

        self.templates
            .iter()
            .map(|template| {
                let handlebars = Handlebars::new();
                let rendered_content = handlebars
                    .render_template(&template.get_content(), &map)
                    .map_err(|e| PromptError::RenderError(e.to_string()))?;

                match template.get_type().as_str() {
                    "user" => Ok(Box::new(HumanMessage {
                        content: rendered_content,
                    }) as Box<dyn BaseMessage>),
                    "system" => Ok(Box::new(SystemMessage {
                        content: rendered_content,
                    }) as Box<dyn BaseMessage>),
                    "assistant" => Ok(Box::new(AIMessage {
                        content: rendered_content,
                    }) as Box<dyn BaseMessage>),
                    _ => Err(PromptError::RenderError(
                        "Unknown message type.".to_string(),
                    )),
                }
            })
            .collect()
    }

    pub fn render_from_hashmap(
        &self,
        data: HashMap<String, String>,
    ) -> Result<Vec<Box<dyn BaseMessage>>, PromptError> {
        self.templates
            .iter()
            .map(|template| {
                let handlebars = Handlebars::new();
                let rendered_content = handlebars
                    .render_template(&template.get_content(), &data)
                    .map_err(|e| PromptError::RenderError(e.to_string()))?;

                match template.get_type().as_str() {
                    "user" => Ok(Box::new(HumanMessage {
                        content: rendered_content,
                    }) as Box<dyn BaseMessage>),
                    "system" => Ok(Box::new(SystemMessage {
                        content: rendered_content,
                    }) as Box<dyn BaseMessage>),
                    "assistant" => Ok(Box::new(AIMessage {
                        content: rendered_content,
                    }) as Box<dyn BaseMessage>),
                    _ => Err(PromptError::RenderError(
                        "Unknown message type.".to_string(),
                    )),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::schemas::messages::{HumanMessage, SystemMessage};

    use super::*;

    #[test]
    fn test_render_from_vec() {
        let template = "Hello {{name}} and {{test}}!";
        let vec_data = vec!["Alice".to_string(), "Bob".to_string()];
        let result = PromptTemplate::new_from_vec(template, vec_data);
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
        let result = PromptTemplate::new_from_hashmap(template, map_data);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "Hello Charlie and David!");
    }

    #[test]
    fn test_render_from_vec_messages() {
        // Create BaseMessage instances
        let message1: Box<dyn BaseMessage> =
            Box::new(HumanMessage::new("Hola {{name}}!".to_string()));

        let message2: Box<dyn BaseMessage> = Box::new(HumanMessage::new(
            "Tengo ganas de comer {{food}}.".to_string(),
        ));

        let templates = PromptTemplates::from_messages(vec![message1, message2]);

        let results = templates
            .render_from_vec(vec!["Luis".to_string(), "pizza".to_string()])
            .unwrap();

        assert_eq!(results[0].get_content(), "Hola Luis!");
        // This one will not pass with the current implementation
        // because the vec data doesn't map correctly to the placeholder.
        assert_eq!(results[1].get_content(), "Tengo ganas de comer pizza.");
    }

    #[test]
    fn test_render_from_hashmap_messages() {
        // Create BaseMessage instances
        let message1: Box<dyn BaseMessage> =
            Box::new(SystemMessage::new("Hola {{name}}!".to_string()));
        let message2: Box<dyn BaseMessage> = Box::new(HumanMessage::new(
            "Tengo ganas de comer {{food}}.".to_string(),
        ));

        let templates = PromptTemplates::from_messages(vec![message1, message2]);

        let mut data = HashMap::new();
        data.insert("food".to_string(), "pizza".to_string());
        data.insert("name".to_string(), "Luis".to_string());

        let results = templates.render_from_hashmap(data).unwrap();

        assert_eq!(results[0].get_content(), "Hola Luis!");
        assert_eq!(results[1].get_content(), "Tengo ganas de comer pizza.");
    }
}
