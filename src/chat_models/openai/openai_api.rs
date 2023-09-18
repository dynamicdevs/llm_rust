use super::message_type::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct ApiRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: f32,
}

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub choices: Vec<ApiChoice>,
    pub usage: ApiUsage,
}

#[derive(Deserialize, Debug)]
pub struct ApiChoice {
    pub index: u8,
    pub message: HashMap<String, String>,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
