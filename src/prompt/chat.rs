use serde_json::Value;
use std::{collections::HashMap, error::Error};

use crate::schemas::{
    messages::{
        is_base_message, message_from_map, AIMessage, BaseMessage, ChatMessage, HumanMessage,
        SystemMessage,
    },
    prompt::PromptValue,
};

use super::{BasePromptTemplate, PromptTemplate, TemplateArgs};

pub struct MessagesPlaceholder {
    variable_name: String,
}
impl MessagesPlaceholder {
    pub fn new(variable_name: &str) -> Self {
        Self {
            variable_name: variable_name.to_string(),
        }
    }
}
impl BaseMessagePromptTemplate for MessagesPlaceholder {
    fn format(&self, _args: &dyn TemplateArgs) -> Result<Box<dyn BaseMessage>, Box<dyn Error>> {
        unimplemented!()
    }

    fn format_messages(
        &self,
        args: &dyn TemplateArgs,
    ) -> Result<Vec<Box<dyn BaseMessage>>, Box<dyn Error>> {
        // Retrieve the variable from args
        let map = args.to_map(&self.input_variables())?;
        let value = map.get(&self.variable_name).ok_or_else(|| {
            Box::<dyn Error>::from(format!(
                "Variable '{}' not found in provided arguments",
                self.variable_name
            ))
        })?;

        if let Value::Array(values) = value {
            let mut messages = Vec::new();
            for v in values {
                if is_base_message(v) {
                    let msg_map: HashMap<String, String> = v
                        .as_object()
                        .unwrap()
                        .iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect();
                    messages.push(message_from_map(msg_map).map_err(|e| e.to_string())?)
                } else {
                    return Err(Box::<dyn Error>::from(format!(
                        "Variable '{}' should be a list of base messages, got {:?}",
                        self.variable_name, v
                    )));
                }
            }
            Ok(messages)
        } else {
            Err(Box::<dyn Error>::from(format!(
                "Variable '{}' should be a list of base messages, got {:?}",
                self.variable_name, value
            )))
        }
    }

    fn input_variables(&self) -> Vec<String> {
        vec![self.variable_name.clone()]
    }
}

pub trait BaseMessagePromptTemplate: Send + Sync {
    fn format(&self, args: &dyn TemplateArgs) -> Result<Box<dyn BaseMessage>, Box<dyn Error>>;

    fn format_messages(
        &self,
        args: &dyn TemplateArgs,
    ) -> Result<Vec<Box<dyn BaseMessage>>, Box<dyn Error>> {
        Ok(vec![self.format(args)?])
    }

    fn input_variables(&self) -> Vec<String>;
}

pub struct ChatMessagePromptTemplate {
    role: String,
    prompt: PromptTemplate,
}
impl ChatMessagePromptTemplate {
    pub fn new(role: &str, prompt: &str) -> Self {
        Self {
            role: role.to_string(),
            prompt: PromptTemplate::from_template(prompt),
        }
    }
}
impl BaseMessagePromptTemplate for ChatMessagePromptTemplate {
    fn format(&self, args: &dyn TemplateArgs) -> Result<Box<dyn BaseMessage>, Box<dyn Error>> {
        let text = self.prompt.format(args)?;
        Ok(Box::new(ChatMessage::new(&self.role, &text)))
    }

    fn input_variables(&self) -> Vec<String> {
        self.prompt.input_variables.clone()
    }
}

pub struct HumanMessagePromptTemplate {
    prompt: PromptTemplate,
}
impl HumanMessagePromptTemplate {
    pub fn new(prompt: PromptTemplate) -> Self {
        Self { prompt }
    }
}
impl BaseMessagePromptTemplate for HumanMessagePromptTemplate {
    fn format(&self, args: &dyn TemplateArgs) -> Result<Box<dyn BaseMessage>, Box<dyn Error>> {
        let text = self.prompt.format(args)?;
        Ok(Box::new(HumanMessage::new(&text)))
    }
    fn input_variables(&self) -> Vec<String> {
        self.prompt.input_variables.clone()
    }
}

pub struct AIMessagePromptTemplate {
    prompt: PromptTemplate,
}
impl AIMessagePromptTemplate {
    pub fn new(prompt: PromptTemplate) -> Self {
        Self { prompt }
    }
}
impl BaseMessagePromptTemplate for AIMessagePromptTemplate {
    fn format(&self, args: &dyn TemplateArgs) -> Result<Box<dyn BaseMessage>, Box<dyn Error>> {
        let text = self.prompt.format(args)?;
        Ok(Box::new(AIMessage::new(&text)))
    }

    fn input_variables(&self) -> Vec<String> {
        self.prompt.input_variables.clone()
    }
}

pub struct SystemMessagePromptTemplate {
    prompt: PromptTemplate,
}
impl SystemMessagePromptTemplate {
    pub fn new(prompt: PromptTemplate) -> Self {
        Self { prompt }
    }
}
impl BaseMessagePromptTemplate for SystemMessagePromptTemplate {
    fn format(&self, args: &dyn TemplateArgs) -> Result<Box<dyn BaseMessage>, Box<dyn Error>> {
        let text = self.prompt.format(args)?;
        Ok(Box::new(SystemMessage::new(&text)))
    }

    fn input_variables(&self) -> Vec<String> {
        self.prompt.input_variables.clone()
    }
}

pub struct ChatPromptValue {
    messages: Vec<Box<dyn BaseMessage>>,
}
impl PromptValue for ChatPromptValue {
    fn to_string(&self) -> Result<String, Box<dyn Error>> {
        let mut text = String::new();
        for msg in &self.messages {
            text.push_str(&format!("{}:{}\n", &msg.get_type(), &msg.get_content()))
        }
        Ok(text)
    }
    fn to_chat_messages(
        &self,
    ) -> Result<Vec<Box<dyn crate::schemas::messages::BaseMessage>>, Box<dyn Error>> {
        Ok(self.messages.clone())
    }
}

pub trait BaseChatPromptTemplate: Send + Sync {
    fn format(&self, args: &dyn TemplateArgs) -> Result<String, Box<dyn Error>> {
        self.format_prompt(args)?.to_string()
    }

    fn format_prompt(
        &self,
        args: &dyn TemplateArgs,
    ) -> Result<Box<dyn PromptValue>, Box<dyn Error>> {
        let messages = self.format_messages(args)?;
        Ok(Box::new(ChatPromptValue { messages }))
    }

    fn format_messages(
        &self,
        args: &dyn TemplateArgs,
    ) -> Result<Vec<Box<dyn BaseMessage>>, Box<dyn Error>>;

    fn input_variables(&self) -> Vec<String>;
}

pub enum MessageLike {
    BaseMessagePromptTemplate(Box<dyn BaseMessagePromptTemplate>),
    BaseMessage(Box<dyn BaseMessage>),
    BaseChatPromptTemplate(Box<dyn BaseChatPromptTemplate>),
}

impl MessageLike {
    pub fn base_message<M: 'static + BaseMessage>(msg: M) -> Self {
        MessageLike::BaseMessage(Box::new(msg))
    }

    pub fn base_prompt_template<T: 'static + BaseMessagePromptTemplate>(template: T) -> Self {
        MessageLike::BaseMessagePromptTemplate(Box::new(template))
    }

    pub fn base_chat_prompt_template<T: 'static + BaseChatPromptTemplate>(template: T) -> Self {
        MessageLike::BaseChatPromptTemplate(Box::new(template))
    }
}

pub struct ChatPromptTemplate {
    input_variables: Vec<String>,
    partial_variables: Option<HashMap<String, Value>>,
    messages: Vec<MessageLike>,
}

impl ChatPromptTemplate {
    pub fn from_template(template: &str) -> Self {
        let prompt_template = PromptTemplate::from_template(template);
        let message = HumanMessagePromptTemplate::new(prompt_template);

        ChatPromptTemplate::from_messages(vec![MessageLike::BaseMessagePromptTemplate(Box::new(
            message,
        ))])
    }

    pub fn from_messages(messages: Vec<MessageLike>) -> Self {
        let mut input_variables = Vec::new();
        for message in &messages {
            match message {
                MessageLike::BaseMessagePromptTemplate(message) => {
                    input_variables.extend(message.input_variables());
                }
                MessageLike::BaseChatPromptTemplate(message) => {
                    input_variables.extend(message.input_variables());
                }
                _ => (),
            }
        }
        Self {
            input_variables,
            messages,
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

    fn merge_partial_and_user_variables(
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

impl BaseChatPromptTemplate for ChatPromptTemplate {
    fn format_messages(
        &self,
        args: &dyn TemplateArgs,
    ) -> Result<Vec<Box<dyn BaseMessage>>, Box<dyn Error>> {
        let merged_args = args.to_map(&self.input_variables)?;
        for var in &self.input_variables {
            if !merged_args.contains_key(var) {
                return Err(Box::new(std::fmt::Error));
            }
        }
        let merged = self.merge_partial_and_user_variables(&merged_args);
        let mut result: Vec<Box<dyn BaseMessage>> = Vec::new();
        for message in &self.messages {
            match message {
                MessageLike::BaseMessagePromptTemplate(message) => {
                    let rel_params: HashMap<String, Value> = merged
                        .iter()
                        .filter(|&(key, _)| message.input_variables().contains(key))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    result.extend(message.format_messages(&rel_params)?)
                }
                MessageLike::BaseChatPromptTemplate(message) => {
                    let rel_params: HashMap<String, Value> = merged
                        .iter()
                        .filter(|&(key, _)| message.input_variables().contains(key))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    result.extend(message.format_messages(&rel_params)?)
                }
                MessageLike::BaseMessage(message) => result.push(message.clone()),
            }
        }

        Ok(result)
    }

    fn input_variables(&self) -> Vec<String> {
        self.input_variables.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_chatprompt_from_messages() {
        let chat_prompt =
            ChatPromptTemplate::from_messages(vec![MessageLike::BaseMessagePromptTemplate(
                Box::new(HumanMessagePromptTemplate::new(
                    PromptTemplate::from_template("Hello, {{name}}!"),
                )),
            )]);

        assert_eq!(chat_prompt.input_variables, vec!["name"]);
        assert_eq!(chat_prompt.partial_variables, None);
    }

    #[test]
    fn test_chatprompt_with_partial_variables() {
        let partial_vars = {
            let mut map = HashMap::new();
            map.insert("city".to_string(), json!("NY"));
            map
        };

        let chat_prompt =
            ChatPromptTemplate::from_messages(vec![MessageLike::BaseMessagePromptTemplate(
                Box::new(HumanMessagePromptTemplate::new(
                    PromptTemplate::from_template("Hello, {{name}} from {{city}}!"),
                )),
            )])
            .with_partial_variables(partial_vars);

        let mut user_vars = HashMap::new();
        user_vars.insert("name".to_string(), Value::String("Alice".to_string()));
        user_vars.insert("city".to_string(), Value::String("London".to_string()));
        println!(
            "{:?}",
            chat_prompt.format_messages(&user_vars).unwrap()[0].get_content()
        );
        assert_eq!(chat_prompt.input_variables, vec!["name"]);
    }

    #[test]
    fn test_chatprompt_format_messages_with_partial_and_user_vars() {
        let partial_vars = {
            let mut map = HashMap::new();
            map.insert("city".to_string(), json!("NY"));
            map
        };

        let chat_prompt =
            ChatPromptTemplate::from_messages(vec![MessageLike::BaseMessagePromptTemplate(
                Box::new(HumanMessagePromptTemplate::new(
                    PromptTemplate::from_template("Hello, {{name}} from {{city}}!"),
                )),
            )])
            .with_partial_variables(partial_vars);

        let messages = chat_prompt.format_messages(&String::from("Alice")).unwrap();
        for message in &messages {
            println!("{:?}", message.get_content())
        }
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].get_content(), "Hello, Alice from NY!");
    }

    #[test]
    fn test_chatprompt_format_messages_with_unmatched_vars() {
        let chat_prompt =
            ChatPromptTemplate::from_messages(vec![MessageLike::BaseMessagePromptTemplate(
                Box::new(HumanMessagePromptTemplate::new(
                    PromptTemplate::from_template("Hello, {{name}}!"),
                )),
            )]);

        let user_vars = HashMap::new();
        let result = chat_prompt.format_messages(&user_vars);

        assert!(result.is_err());
    }

    #[test]
    fn test_all() {
        let prompt = ChatPromptTemplate::from_messages(vec![
            MessageLike::base_message(SystemMessage::new("hola este es el system message")),
            MessageLike::base_prompt_template(MessagesPlaceholder::new("chat_history")),
            MessageLike::base_prompt_template(HumanMessagePromptTemplate::new(
                PromptTemplate::from_template("el final prompt hecho por {{name}}"),
            )),
            MessageLike::base_prompt_template(MessagesPlaceholder::new("sratch_pad")),
        ]);
        let user_vars = {
            let mut map = HashMap::new();
            let mut chat_history = Vec::new();
            let msg = HumanMessage::new("Hello, world!");
            chat_history.push(Box::new(msg) as Box<dyn BaseMessage>);
            map.insert("name".to_string(), json!("Alice"));
            map.insert("chat_history".to_string(), json!(chat_history));
            let sratch: Vec<Box<dyn BaseMessage>> = vec![
                // Box::new(HumanMessage::new("test1")) as Box<dyn BaseMessage>,
                // Box::new(HumanMessage::new("test2")) as Box<dyn BaseMessage>,
            ];
            map.insert("sratch_pad".to_string(), json!(sratch));
            map
        };

        let messages = prompt.format_messages(&user_vars).unwrap();
        for message in &messages {
            println!("{:?}", message.get_content())
        }
        assert!(prompt.format_messages(&user_vars).is_ok())
    }
}
