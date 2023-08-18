use crate::schemas::messages::BaseMessage;

pub trait BasePromptValue {
    fn to_string(&self) -> String;
    fn to_chat_messages(&self) -> Vec<Box<dyn BaseMessage>>;
}
