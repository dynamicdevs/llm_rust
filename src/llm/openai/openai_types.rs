use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    User,
    System,
    Assistant,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match *self {
            Role::User => "User",
            Role::System => "System",
            Role::Assistant => "Assistant",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    role: Role,
    content: String,
}
impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    messages: Vec<Message>,
}
impl Messages {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(Message::new(Role::User, content));
    }

    pub fn add_system_message(&mut self, content: String) {
        self.messages.push(Message::new(Role::System, content));
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(Message::new(Role::Assistant, content));
    }
}
