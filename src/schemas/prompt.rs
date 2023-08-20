use std::collections::HashMap;

use crate::{errors::PromptError, schemas::messages::BaseMessage};

pub enum PromptData {
    VecData(Vec<String>),
    HashMapData(HashMap<String, String>),
}

pub trait BasePromptValue: Send + Sync {
    fn to_string(&self) -> Result<String, PromptError>;
    fn to_chat_messages(&self) -> Result<Vec<Box<dyn BaseMessage>>, PromptError>;
    fn add_values(&mut self, data: PromptData);
}
