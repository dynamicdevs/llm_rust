use std::{
    collections::HashMap,
    io::{self, ErrorKind},
};

pub trait BaseMessage: Send {
    fn get_type(&self) -> String;
    fn get_content(&self) -> String;
}

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
        String::from("human")
    }

    fn get_content(&self) -> String {
        self.content.clone()
    }
}

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
}

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
        String::from("ai")
    }

    fn get_content(&self) -> String {
        self.content.clone()
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
        "human" => {
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

        "ai" => {
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

fn message_to_map(message: Box<dyn BaseMessage>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    map.insert("type".to_string(), message.get_type());
    map.insert("content".to_string(), message.get_content());

    map
}

pub fn messages_to_map(messages: Vec<Box<dyn BaseMessage>>) -> Vec<HashMap<String, String>> {
    messages.into_iter().map(message_to_map).collect()
}