use crate::schemas::{base_prompt_value::BasePromptValue, messages::BaseMessage};

pub struct ChatPromptValue {
    messages: Vec<Box<dyn BaseMessage>>,
}
impl ChatPromptValue {
    pub fn new(messages: Vec<Box<dyn BaseMessage>>) -> Self {
        Self { messages }
    }
}
impl BasePromptValue for ChatPromptValue {
    fn to_string(&self) -> String {
        let mut string_prompt = String::new();
        self.messages.iter().for_each(|message| {
            string_prompt.push_str(&format!(
                "{}:{}\n",
                message.get_type(),
                message.get_content()
            ));
        });
        string_prompt
    }

    fn to_chat_messages(&self) -> Vec<Box<dyn BaseMessage>> {
        self.messages.clone()
    }
}
