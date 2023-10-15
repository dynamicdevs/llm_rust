use crate::schemas::{messages::HumanMessage, prompt::PromptValue};
use handlebars::Handlebars;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use super::TemplateArgs;

pub trait BasePromptTemplate {
    fn format(&self, args: &dyn TemplateArgs) -> Result<String, Box<dyn Error>>;
    fn format_prompt(
        &self,
        args: &dyn TemplateArgs,
    ) -> Result<Box<dyn PromptValue>, Box<dyn Error>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    template: String,
    pub input_variables: Vec<String>,
    partial_variables: Option<HashMap<String, Value>>,
}

fn extract_handlebars_input_variables(template: &str) -> Vec<String> {
    let re = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").unwrap();
    let mut input_variables_set = HashSet::new();

    for cap in re.captures_iter(template) {
        if let Some(matched) = cap.get(1) {
            input_variables_set.insert(matched.as_str().to_string());
        }
    }

    let mut input_variables: Vec<String> = input_variables_set.into_iter().collect();
    input_variables.sort();
    input_variables
}

impl PromptTemplate {
    pub fn from_template(template: &str) -> Self {
        let input_vars = extract_handlebars_input_variables(template);

        Self {
            template: template.to_string(),
            input_variables: input_vars,
            partial_variables: None,
        }
    }

    pub fn with_partial_variables(mut self, partial_variables: HashMap<String, Value>) -> Self {
        for key in partial_variables.keys() {
            self.input_variables.retain(|var| var != key);
        }
        self.partial_variables = Some(partial_variables);
        self
    }

    pub fn merge_partial_and_user_variables(
        &self,
        user_variables: &HashMap<String, Value>,
    ) -> HashMap<String, Value> {
        let mut merged = HashMap::new();

        if let Some(ref partial) = self.partial_variables {
            for (k, v) in partial {
                merged.insert(k.clone(), v.clone());
            }
        }

        for (k, v) in user_variables {
            merged.insert(k.clone(), v.clone());
        }

        merged
    }
}

impl BasePromptTemplate for PromptTemplate {
    fn format(&self, args: &dyn TemplateArgs) -> Result<String, Box<dyn Error>> {
        let merged_args = args.to_map(&self.input_variables)?;
        for var in &self.input_variables {
            if !merged_args.contains_key(var) {
                return Err(Box::new(std::fmt::Error));
            }
        }
        let merged = self.merge_partial_and_user_variables(&merged_args);
        let handlebars = Handlebars::new();
        let prompt = handlebars.render_template(&self.template, &merged)?;
        Ok(prompt)
    }

    fn format_prompt(
        &self,
        args: &dyn TemplateArgs,
    ) -> Result<Box<dyn PromptValue>, Box<dyn Error>> {
        Ok(Box::new(StringPromptValue {
            text: self.format(args)?,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringPromptValue {
    text: String,
}

impl PromptValue for StringPromptValue {
    fn to_string(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.text.clone())
    }

    fn to_chat_messages(
        &self,
    ) -> Result<Vec<Box<dyn crate::schemas::messages::BaseMessage>>, Box<dyn Error>> {
        Ok(vec![Box::new(HumanMessage::new(&self.text))])
    }
}

pub struct StringPromptTemplate {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_template() {
        let template_str = "Hello, {{name}}!";
        let template = PromptTemplate::from_template(template_str);

        assert_eq!(template.template, template_str);
        assert_eq!(template.input_variables, vec!["name"]);
        assert_eq!(template.partial_variables, None);
    }

    #[test]
    fn test_with_partial_variables() {
        let template_str = "Hello, {{name}} from {{city}}!";
        let mut partial_vars = HashMap::new();
        partial_vars.insert("city".to_string(), Value::String("NY".to_string()));

        let template =
            PromptTemplate::from_template(template_str).with_partial_variables(partial_vars);

        assert_eq!(template.input_variables, vec!["name"]);
    }

    #[test]
    fn test_format_with_partial_and_user_vars() {
        let template_str = "Hello, {{name}} from {{city}}!";
        let mut partial_vars = HashMap::new();
        partial_vars.insert("city".to_string(), Value::String("NY".to_string()));

        let template =
            PromptTemplate::from_template(template_str).with_partial_variables(partial_vars);

        let mut user_vars = HashMap::new();
        user_vars.insert("name".to_string(), Value::String("Alice".to_string()));

        let output = template.format(&user_vars).unwrap();

        assert_eq!(output, "Hello, Alice from NY!");
    }

    #[test]
    fn test_format_with_unmatched_vars() {
        let template_str = "Hello, {{name}}!";
        let template = PromptTemplate::from_template(template_str);

        let user_vars = HashMap::new();

        let result = template.format(&user_vars);
        println!("{:?}", result);

        assert!(result.is_err());
    }

    #[test]
    fn test_format_with_string_template_args() {
        let template_str = "Hello, {{input}}!";
        let template = PromptTemplate::from_template(template_str);
        let name = "Alice".to_string();

        let output = template.format(&name).unwrap();

        assert_eq!(output, "Hello, Alice!");
    }
}
