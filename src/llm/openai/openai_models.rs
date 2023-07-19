#[derive(Debug)]
pub enum ChatModel {
    Gpt3_5Turbo,
    Gpt3_5Turbo16k,
}
impl ChatModel {
    pub fn as_str(&self) -> &str {
        match *self {
            ChatModel::Gpt3_5Turbo => "gpt-3.5-turbo",
            ChatModel::Gpt3_5Turbo16k => "gpt-3.5-turbo-16k",
        }
    }
}
