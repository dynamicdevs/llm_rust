use std::env;

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    chat_models::{
        chat_model_trait::ChatTrait,
        openai::{
            message_type::Message,
            openai_api::{ApiRequest, ApiResponse},
        },
    },
    errors::{openai_errors::OpenaiError, ApiError},
    schemas::messages::{AIMessage, BaseMessage},
};

#[derive(Debug)]
pub enum ChatModel {
    Gpt3_5Turbo,
    Gpt3_5Turbo16k,
    GPT3_5TURBO0613,
    Gpt4,
}
impl ChatModel {
    pub fn as_str(&self) -> &str {
        match *self {
            ChatModel::Gpt3_5Turbo => "gpt-3.5-turbo",
            ChatModel::Gpt3_5Turbo16k => "gpt-3.5-turbo-16k",
            ChatModel::GPT3_5TURBO0613 => "gpt-3.5-turbo-0613",
            ChatModel::Gpt4 => "gpt-4",
        }
    }
}

pub struct ChatOpenAI {
    pub model: ChatModel,
    pub temperature: f32,
    pub openai_key: String,
    pub max_tokens: Option<u32>,
}
impl ChatOpenAI {
    pub fn new(model: ChatModel, temperature: f32, openai_key: String) -> Self {
        Self {
            model,
            temperature,
            openai_key,
            max_tokens: None,
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

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}
impl Default for ChatOpenAI {
    fn default() -> Self {
        Self {
            model: ChatModel::Gpt3_5Turbo,
            temperature: 0.0,
            openai_key: env::var("OPENAI_API_KEY").unwrap_or(String::new()),
            max_tokens: None,
        }
    }
}
#[async_trait]
impl ChatTrait for ChatOpenAI {
    async fn generate(
        &self,
        messages: Vec<Vec<Box<dyn BaseMessage>>>,
    ) -> Result<AIMessage, ApiError> {
        let flattened_messages: Vec<Message> = messages
            .into_iter()
            .flat_map(|inner_messages| Message::from_base_messages(inner_messages))
            .collect();
        log::debug!("flattened_messages: {:?}", flattened_messages);

        let client = Client::new();
        let api_request = ApiRequest {
            model: String::from(self.model.as_str()),
            messages: flattened_messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai_key))
            .json(&api_request)
            .send()
            .await
            .map_err(|_| {
                ApiError::OpenaiError(OpenaiError::new_generic_error(String::from(
                    "Error deserializing response or unknow error",
                )))
            })?;

        let status = response.status();
        match response.status() {
            reqwest::StatusCode::OK => {
                let api_response: ApiResponse = response.json().await.map_err(|_| {
                    ApiError::OpenaiError(OpenaiError::new_generic_error(String::from(
                        "Error deserializing response or unknow error",
                    )))
                })?;
                log::info!(
                    "Prompt token count: {:?} for madel: {}",
                    api_response.usage,
                    self.model.as_str()
                );
                let response_message = api_response
                    .choices
                    .get(0)
                    .ok_or_else(|| OpenaiError::ServerError {
                        code: 500,
                        detail: String::from("Unexpected API response"),
                    })
                    .map_err(|e| ApiError::OpenaiError(e.clone()))?;

                let ai_message = AIMessage::new(
                    &response_message
                        .message
                        .get("content")
                        .ok_or_else(|| OpenaiError::ServerError {
                            code: 500,
                            detail: String::from("No content in AI message"),
                        })
                        .map_err(|e| ApiError::OpenaiError(e.clone()))?
                        .clone(),
                );
                Ok(ai_message)
            }
            _ => {
                let detail: String = response.text().await.unwrap();
                let error =
                    OpenaiError::from_http_status(status.as_u16().clone(), detail.to_string());
                Err(ApiError::OpenaiError(error))
            }
        }
    }
}
