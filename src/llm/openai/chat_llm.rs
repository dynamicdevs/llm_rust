use std::env;

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde_json::{json, Value};

use crate::{
    errors::{openai_errors::OpenaiError, ApiError},
    llm::llm_trait::LLM,
};

use super::{openai_models::ChatModel, openai_types::Messages};

#[derive(Debug)]
pub struct ChatLLM {
    pub model: ChatModel,
    pub temperature: u32,
    pub messages: Messages,
    pub openai_key: String,
}
impl ChatLLM {
    pub fn new(model: ChatModel, temperature: u32, messages: Messages, openai_key: String) -> Self {
        Self {
            model,
            temperature,
            messages,
            openai_key,
        }
    }

    pub fn with_model(mut self, model: ChatModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_api_key(mut self, openai_key: String) -> Self {
        self.openai_key = openai_key;
        self
    }

    pub fn with_temperature(mut self, temperature: u32) -> Self {
        self.temperature = temperature;
        self
    }
}

impl Default for ChatLLM {
    fn default() -> Self {
        Self {
            model: ChatModel::Gpt3_5Turbo,
            temperature: 0,
            messages: Messages::new(),
            openai_key: env::var("OPENAI_API_KEY").unwrap_or(String::new()),
        }
    }
}

#[async_trait]
impl LLM for ChatLLM {
    async fn generate_completition(&mut self, text: &str) -> Result<String, ApiError> {
        let client = Client::new();
        let url = Url::parse("https://api.openai.com/v1/chat/completions").unwrap();

        self.messages.add_user_message(text.to_string());
        let body = json!({
            "model": self.model.as_str(),
            "temperature": self.temperature,
            "messages": self.messages,
        });

        let res = client
            .post(url)
            .bearer_auth(self.openai_key.as_str())
            .json(&body)
            .send()
            .await;

        match res {
            Ok(response) => {
                let data: Value = response.json().await.unwrap();
                let text = data["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap()
                    .to_string();
                Ok(text)
            }
            Err(err) => Err(ApiError::OpenaiError(OpenaiError::from_http_status(
                err.status().unwrap().as_u16(),
                err.to_string(),
            ))),
        }
    }
}
