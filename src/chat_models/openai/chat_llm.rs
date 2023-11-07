use std::env;

use async_trait::async_trait;
use reqwest::Client;
use reqwest_eventsource::EventSource;

use crate::{
    chat_models::{
        chat_model_trait::ChatTrait,
        openai::{
            message_type::Message,
            openai_api::{ApiRequest, ApiResponse},
        },
    },
    errors::{openai_errors::OpenaiError, ApiError},
    schemas::{llm::LlmResponse, messages::BaseMessage},
};

#[derive(Debug)]
pub enum ChatModel {
    Gpt3_5Turbo,
    Gpt3_5Turbo16k,
    GPT3_5TURBO0613,
    Gpt4,
    Gpt4TURBO,
}
impl ChatModel {
    pub fn as_str(&self) -> &str {
        match *self {
            ChatModel::Gpt3_5Turbo => "gpt-3.5-turbo",
            ChatModel::Gpt3_5Turbo16k => "gpt-3.5-turbo-16k",
            ChatModel::GPT3_5TURBO0613 => "gpt-3.5-turbo-0613",
            ChatModel::Gpt4 => "gpt-4",
            ChatModel::Gpt4TURBO => "gpt-4-1106-preview",
        }
    }
}

pub struct ChatOpenAI {
    pub model: ChatModel,
    pub temperature: f32,
    pub openai_key: String,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}
impl ChatOpenAI {
    pub fn new(model: ChatModel, temperature: f32, openai_key: String) -> Self {
        Self {
            model,
            temperature,
            openai_key,
            max_tokens: None,
            stream: false,
        }
    }

    pub fn with_model(mut self, model: ChatModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_stream(mut self) -> Self {
        self.stream = true;
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
            stream: false,
        }
    }
}

#[async_trait]
impl ChatTrait for ChatOpenAI {
    async fn generate(
        &self,
        messages: Vec<Vec<Box<dyn BaseMessage>>>,
    ) -> Result<LlmResponse, ApiError> {
        let flattened_messages: Vec<Message> = messages
            .into_iter()
            .flat_map(|inner_messages| Message::from_base_messages(inner_messages))
            .collect();
        log::debug!("flattened_messages: {:?}", flattened_messages);

        let client = Client::new();
        let mut api_request = ApiRequest {
            model: String::from(self.model.as_str()),
            messages: flattened_messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            stream: None,
        };

        // Add the 'stream' parameter if streaming is requested
        if self.stream {
            api_request.stream = Some(true);
        }

        let request = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai_key))
            .json(&api_request);

        if self.stream {
            let es = EventSource::new(request).map_err(|e| {
                ApiError::OpenaiError(OpenaiError::new_generic_error(format!(
                    "Error creating EventSource: {}",
                    e
                )))
            })?;
            return Ok(LlmResponse::Stream(es));
        }

        let response = request.send().await.map_err(|e| {
            ApiError::OpenaiError(OpenaiError::new_generic_error(format!(
                "Error sending request: {}",
                e
            )))
        })?;
        let status = response.status();
        match status {
            reqwest::StatusCode::OK => {
                let api_response: ApiResponse = response.json().await.map_err(|_| {
                    ApiError::OpenaiError(OpenaiError::new_generic_error(String::from(
                        "Error deserializing response or unknown error",
                    )))
                })?;
                log::info!(
                    "Prompt token count: {:?} for model: {}",
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

                let text_resp = response_message
                    .message
                    .get("content")
                    .ok_or_else(|| OpenaiError::ServerError {
                        code: 500,
                        detail: String::from("No content in AI message"),
                    })
                    .map_err(|e| ApiError::OpenaiError(e.clone()))?
                    .clone();

                Ok(LlmResponse::Text(text_resp))
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
