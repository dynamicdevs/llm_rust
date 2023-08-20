use handlebars::Handlebars;
use regex::Regex;
use std::collections::HashMap;

use crate::{
    errors::PromptError,
    schemas::{
        messages::{AIMessage, BaseMessage, HumanMessage, SystemMessage},
        prompt::{BasePromptValue, PromptData},
    },
};

pub struct PromptTemplate {
    template: String,
    data: Option<PromptData>,
}

impl PromptTemplate {
    pub fn new(template: &str) -> Self {
        PromptTemplate {
            template: template.to_string(),
            data: None,
        }
    }

    pub fn new_from_vec(template: &str, data: Vec<String>) -> Self {
        PromptTemplate {
            template: template.to_string(),
            data: Some(PromptData::VecData(data)),
        }
    }

    pub fn new_from_hashmap(template: &str, data: HashMap<String, String>) -> Self {
        PromptTemplate {
            template: template.to_string(),
            data: Some(PromptData::HashMapData(data)),
        }
    }

    pub fn render(&self) -> Result<String, PromptError> {
        let re = Regex::new(r"\{\{([^\{\}]+)\}\}").unwrap();
        let captures: Vec<String> = re
            .captures_iter(&self.template)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .collect();

        let mut map = HashMap::new();
        match &self.data {
            Some(PromptData::VecData(data_vec)) => {
                for (index, placeholder) in captures.iter().enumerate() {
                    if let Some(value) = data_vec.get(index) {
                        map.insert(placeholder.clone(), value.clone());
                    }
                }
            }
            Some(PromptData::HashMapData(data_map)) => {
                map.extend(data_map.clone());
            }
            None => return Err(PromptError::DataNotProvided("No data".to_string())),
        }

        let handlebars = Handlebars::new();
        handlebars
            .render_template(&self.template, &map)
            .map_err(|e| PromptError::RenderError(e.to_string()))
    }
}

impl BasePromptValue for PromptTemplate {
    fn to_string(&self) -> Result<String, PromptError> {
        self.render()
    }

    fn to_chat_messages(&self) -> Result<Vec<Box<dyn BaseMessage>>, PromptError> {
        Ok(vec![Box::new(HumanMessage::new(self.render()?))])
    }

    fn add_values(&mut self, data: PromptData) {
        self.data = Some(data);
    }
}

pub struct PromptTemplates {
    templates: Vec<Box<dyn BaseMessage>>,
    data: Option<PromptData>,
}

impl PromptTemplates {
    pub fn new(templates: Vec<Box<dyn BaseMessage>>) -> Self {
        PromptTemplates {
            templates,
            data: None,
        }
    }

    fn render(&self) -> Result<Vec<Box<dyn BaseMessage>>, PromptError> {
        match &self.data {
            Some(PromptData::VecData(data_vec)) => self.render_from_vec(data_vec.clone()),
            Some(PromptData::HashMapData(data_hashmap)) => {
                self.render_from_hashmap(data_hashmap.clone())
            }
            None => Err(PromptError::RenderError("No data provided.".to_string())),
        }
    }

    fn render_from_vec(&self, data: Vec<String>) -> Result<Vec<Box<dyn BaseMessage>>, PromptError> {
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

    fn render_from_hashmap(
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

impl BasePromptValue for PromptTemplates {
    fn to_string(&self) -> Result<String, PromptError> {
        self.render()
            .map(|messages| messages.iter().map(|m| m.get_content()).collect())
    }

    fn to_chat_messages(&self) -> Result<Vec<Box<dyn BaseMessage>>, PromptError> {
        match &self.data {
            Some(PromptData::VecData(data_vec)) => self.render_from_vec(data_vec.clone()),
            Some(PromptData::HashMapData(data_hashmap)) => {
                self.render_from_hashmap(data_hashmap.clone())
            }
            None => Err(PromptError::RenderError("No data provided.".to_string())),
        }
    }

    fn add_values(&mut self, data: PromptData) {
        self.data = Some(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::messages::{HumanMessage, SystemMessage};

    #[test]
    fn test_render_from_vec() {
        let template = "Hello {{name}} and {{test}}!";
        let vec_data = vec!["Alice".to_string(), "Bob".to_string()];
        let prompt = PromptTemplate::new_from_vec(template, vec_data);
        let result = prompt.render();
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
        let prompt = PromptTemplate::new_from_hashmap(template, map_data);
        let result = prompt.render();
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "Hello Charlie and David!");
    }

    #[test]
    fn test_render_from_vec_messages() {
        let message1: Box<dyn BaseMessage> =
            Box::new(HumanMessage::new("Hola {{name}}!".to_string()));
        let message2: Box<dyn BaseMessage> = Box::new(HumanMessage::new(
            "Tengo ganas de comer {{food}}.".to_string(),
        ));

        let mut templates = PromptTemplates::new(vec![message1, message2]);
        templates.add_values(PromptData::VecData(vec![
            "Luis".to_string(),
            "pizza".to_string(),
        ]));

        let results = templates.to_chat_messages().unwrap();

        assert_eq!(results[0].get_content(), "Hola Luis!");
        assert_eq!(results[1].get_content(), "Tengo ganas de comer pizza.");
    }

    #[test]
    fn test_render_from_hashmap_messages() {
        let message1: Box<dyn BaseMessage> =
            Box::new(SystemMessage::new("Hola {{name}}!".to_string()));
        let message2: Box<dyn BaseMessage> = Box::new(HumanMessage::new(
            "Tengo ganas de comer {{food}}.".to_string(),
        ));

        let mut data = HashMap::new();
        data.insert("food".to_string(), "pizza".to_string());
        data.insert("name".to_string(), "Luis".to_string());

        let mut templates = PromptTemplates::new(vec![message1, message2]);
        templates.add_values(PromptData::HashMapData(data));

        let results = templates.to_chat_messages().unwrap();

        assert_eq!(results[0].get_content(), "Hola Luis!");
        assert_eq!(results[1].get_content(), "Tengo ganas de comer pizza.");
    }

    #[test]
    fn test_render_prompt_template_new_with_add_values() {
        let template = "Hi {{greeting}}!";
        let mut prompt = PromptTemplate::new(template);
        prompt.add_values(PromptData::HashMapData({
            let mut map = HashMap::new();
            map.insert("greeting".to_string(), "there".to_string());
            map
        }));
        let result = prompt.render();
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "Hi there!");
    }

    #[test]
    fn test_render_prompt_templates_new_with_add_values() {
        let message1: Box<dyn BaseMessage> =
            Box::new(HumanMessage::new("Good {{time_of_day}}!".to_string()));
        let message2: Box<dyn BaseMessage> =
            Box::new(HumanMessage::new("I love eating {{meal}}.".to_string()));

        let mut templates = PromptTemplates::new(vec![message1, message2]);
        templates.add_values(PromptData::HashMapData({
            let mut map = HashMap::new();
            map.insert("time_of_day".to_string(), "morning".to_string());
            map.insert("meal".to_string(), "breakfast".to_string());
            map
        }));

        let results = templates.to_chat_messages().unwrap();

        assert_eq!(results[0].get_content(), "Good morning!");
        assert_eq!(results[1].get_content(), "I love eating breakfast.");
    }

    #[test]
    fn test_render_prompt_template_new_with_add_values_vec() {
        let template = "Hello {{name}}, you are number {{number}}!";
        let mut prompt = PromptTemplate::new(template);
        prompt.add_values(PromptData::VecData(vec![
            "John".to_string(),
            "1".to_string(),
        ]));
        let result = prompt.render();
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered, "Hello John, you are number 1!");
    }

    #[test]
    fn test_render_prompt_templates_new_with_add_values_vec() {
        let message1: Box<dyn BaseMessage> =
            Box::new(HumanMessage::new("{{greeting}} {{name}}!".to_string()));
        let message2: Box<dyn BaseMessage> =
            Box::new(HumanMessage::new("You are {{designation}}.".to_string()));

        let mut templates = PromptTemplates::new(vec![message1, message2]);
        templates.add_values(PromptData::VecData(vec![
            "Hello".to_string(),
            "John".to_string(),
            "the first".to_string(),
        ]));

        let results = templates.to_chat_messages().unwrap();

        assert_eq!(results[0].get_content(), "Hello John!");
        assert_eq!(results[1].get_content(), "You are the first.");
    }
}
