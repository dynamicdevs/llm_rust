use std::env;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::{
    errors::{openai_errors::OpenaiError, ApiError},
    llm::base::BaseLLM,
};

#[derive(Debug)]
pub enum LLMModel {
    GptDavinci002,
    TextDavinci003,
}

impl LLMModel {
    pub fn as_str(&self) -> &str {
        match *self {
            LLMModel::GptDavinci002 => "davinci-002",
            LLMModel::TextDavinci003 => "text-davinci-003",
        }
    }
}

pub struct LLMOpenAI {
    pub model: LLMModel,
    pub temperature: u32,
    pub openai_key: String,
    pub stop_sequence: Option<String>,
}
impl LLMOpenAI {
    pub fn new(model: LLMModel, temperature: u32, openai_key: String) -> Self {
        Self {
            model,
            temperature,
            openai_key,
            stop_sequence: None,
        }
    }

    pub fn with_model(mut self, model: LLMModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_api_key(mut self, openai_key: String) -> Self {
        self.openai_key = openai_key;
        self
    }

    pub fn with_stop_sequence(mut self, stop_sequence: String) -> Self {
        self.stop_sequence = Some(stop_sequence);
        self
    }

    pub fn with_temperature(mut self, temperature: u32) -> Self {
        self.temperature = temperature;
        self
    }
}

impl Default for LLMOpenAI {
    fn default() -> Self {
        Self {
            model: LLMModel::TextDavinci003,
            temperature: 0,
            openai_key: env::var("OPENAI_API_KEY").unwrap_or(String::new()),
            stop_sequence: Some(String::from("\n")),
        }
    }
}

#[derive(Deserialize)]
struct CompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

#[async_trait]
impl BaseLLM for LLMOpenAI {
    async fn generate(&self, prompt: String) -> Result<String, ApiError> {
        let client = Client::new();
        let url = format!(
            "https://api.openai.com/v1/engines/{}/completions",
            self.model.as_str()
        );
        let payload = json!({
            "prompt": prompt,
            "temperature": self.temperature,
            "stop": self.stop_sequence,
        });

        let response = match client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.openai_key))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return Err(ApiError::OpenaiError(OpenaiError::new_generic_error(
                    format!("Unknown error: {}", e),
                )))
            }
        };

        if response.status().is_success() {
            let result: CompletionResponse = match response.json().await {
                Ok(result) => result,
                Err(_) => {
                    return Err(ApiError::OpenaiError(OpenaiError::new_generic_error(
                        String::from("Failed to deserialize JSON"),
                    )))
                }
            };

            match result.choices.get(0) {
                Some(choice) => Ok(choice.text.clone()),
                None => Err(ApiError::OpenaiError(OpenaiError::new_generic_error(
                    String::from("No choices returned"),
                ))),
            }
        } else {
            let code = response.status().as_u16();
            let detail = match response.text().await {
                Ok(detail) => detail,
                Err(_) => String::from("Unknown error"),
            };
            Err(ApiError::OpenaiError(OpenaiError::from_http_status(
                code, detail,
            )))
        }
    }
}
