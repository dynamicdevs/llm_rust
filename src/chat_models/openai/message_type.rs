use serde::{Deserialize, Serialize};

use crate::schemas::messages::BaseMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
impl Message {
    pub fn new(role: String, content: String) -> Self {
        Self { role, content }
    }

    pub fn from_base_message(base: Box<dyn BaseMessage>) -> Self {
        Message {
            role: base.get_type(),
            content: base.get_content(),
        }
    }

    pub fn from_base_messages(messages: Vec<Box<dyn BaseMessage>>) -> Vec<Self> {
        messages
            .into_iter()
            .map(|base| Self {
                role: base.get_type(),
                content: base.get_content(),
            })
            .collect()
    }
}
