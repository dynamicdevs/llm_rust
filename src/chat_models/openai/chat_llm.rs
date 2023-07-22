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
        AIMessage, BaseMessage, HumanMessage,
    },
    errors::{openai_errors::OpenaiError, ApiError},
};

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

pub struct ChatOpenAI {
    pub model: ChatModel,
    pub temperature: u32,
    pub openai_key: String,
}
impl ChatOpenAI {
    pub fn new(model: ChatModel, temperature: u32, openai_key: String) -> Self {
        Self {
            model,
            temperature,
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
impl Default for ChatOpenAI {
    fn default() -> Self {
        Self {
            model: ChatModel::Gpt3_5Turbo,
            temperature: 0,
            openai_key: env::var("OPENAI_API_KEY").unwrap_or(String::new()),
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

        let client = Client::new();
        let api_request = ApiRequest {
            model: String::from(self.model.as_str()),
            messages: flattened_messages,
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
                let api_response: ApiResponse = response.json().await.unwrap();
                let response_message = api_response
                    .choices
                    .get(0)
                    .ok_or_else(|| OpenaiError::ServerError {
                        code: 500,
                        detail: String::from("Unexpected API response"),
                    })
                    .map_err(|e| ApiError::OpenaiError(e.clone()))?;

                let ai_message = AIMessage::new(
                    response_message
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

    async fn call(&self, query: String) -> Result<String, ApiError> {
        let message = HumanMessage::new(query);
        self.generate(vec![vec![Box::new(message)]])
            .await
            .map(|ai_message| ai_message.content)
    }
}
