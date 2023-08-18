use crate::schemas::{
    base_prompt_value::BasePromptValue,
    messages::{BaseMessage, HumanMessage},
};

pub struct StringPromptValue {
    value: String,
}
impl StringPromptValue {
    pub fn new(val: &str) -> Self {
        Self {
            value: String::from(val),
        }
    }
}
impl BasePromptValue for StringPromptValue {
    fn to_string(&self) -> String {
        self.value.clone()
    }
    fn to_chat_messages(&self) -> Vec<Box<dyn BaseMessage>> {
        vec![Box::new(HumanMessage::new(self.value.clone()))]
    }
}
