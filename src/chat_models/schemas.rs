use std::{
    collections::HashMap,
    io::{self, ErrorKind},
};

pub trait BaseMessage: Send + Sync {
    fn get_type(&self) -> String;
    fn get_content(&self) -> String;
    fn clone_box(&self) -> Box<dyn BaseMessage>;
}
impl Clone for Box<dyn BaseMessage> {
    fn clone(&self) -> Box<dyn BaseMessage> {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct HumanMessage {
    pub content: String,
}
impl HumanMessage {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}
impl BaseMessage for HumanMessage {
    fn get_type(&self) -> String {
        String::from("user")
    }

    fn get_content(&self) -> String {
        self.content.clone()
    }
    fn clone_box(&self) -> Box<dyn BaseMessage> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct SystemMessage {
    pub content: String,
}
impl SystemMessage {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}
impl BaseMessage for SystemMessage {
    fn get_type(&self) -> String {
        String::from("system")
    }

    fn get_content(&self) -> String {
        self.content.clone()
    }

    fn clone_box(&self) -> Box<dyn BaseMessage> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct AIMessage {
    pub content: String,
}
impl AIMessage {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}
impl BaseMessage for AIMessage {
    fn get_type(&self) -> String {
        String::from("assistant")
    }

    fn get_content(&self) -> String {
        self.content.clone()
    }

    fn clone_box(&self) -> Box<dyn BaseMessage> {
        Box::new(self.clone())
    }
}

fn message_from_map(
    message: HashMap<String, String>,
) -> Result<Box<dyn BaseMessage>, Box<dyn std::error::Error + Send>> {
    let message_type = match message.get("type") {
        Some(t) => t,
        None => {
            return Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "No type key on map",
            )))
        }
    };

    match message_type.as_str() {
        "user" => {
            let content = message.get("content").unwrap_or(&String::from("")).clone();
            Ok(Box::new(HumanMessage {
                content: content.to_string(),
            }))
        }

        "system" => {
            let content = message.get("content").unwrap_or(&String::from("")).clone();
            Ok(Box::new(SystemMessage {
                content: content.to_string(),
            }))
        }

        "assistant" => {
            let content = message.get("content").unwrap_or(&String::from("")).clone();
            Ok(Box::new(AIMessage {
                content: content.to_string(),
            }))
        }

        _ => Err(Box::new(io::Error::new(
            ErrorKind::Other,
            format!("Got unexpected message type: {}", message_type),
        ))),
    }
}

pub fn messages_from_map(
    messages: Vec<HashMap<String, String>>,
) -> Result<Vec<Box<dyn BaseMessage>>, Box<dyn std::error::Error + Send>> {
    messages.into_iter().map(message_from_map).collect()
}

fn message_to_map<T: BaseMessage>(message: T) -> HashMap<String, String> {
    let mut map = HashMap::new();

    map.insert("type".to_string(), message.get_type());
    map.insert("content".to_string(), message.get_content());

    map
}

pub fn messages_to_map<T: BaseMessage>(messages: Vec<T>) -> Vec<HashMap<String, String>> {
    messages.into_iter().map(message_to_map).collect()
}
