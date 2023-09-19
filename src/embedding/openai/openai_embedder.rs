use std::env;

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde_json::{json, Value};

use crate::{
    embedding::embedder_trait::Embedder,
    errors::{openai_errors::OpenaiError, ApiError},
};

#[derive(Debug)]
pub struct OpenAiEmbedder {
    pub model: String,
    pub openai_key: String,
}
impl OpenAiEmbedder {
    pub fn new(openai_key: String) -> Self {
        OpenAiEmbedder {
            model: String::from("text-embedding-ada-002"),
            openai_key,
        }
    }
}

impl Default for OpenAiEmbedder {
    fn default() -> Self {
        OpenAiEmbedder {
            model: String::from("text-embedding-ada-002"),
            openai_key: env::var("OPENAI_API_KEY").unwrap_or(String::new()),
        }
    }
}

#[async_trait]
impl Embedder for OpenAiEmbedder {
    async fn embed_documents(&self, documents: Vec<String>) -> Result<Vec<Vec<f64>>, ApiError> {
        let client = Client::new();
        let url = Url::parse("https://api.openai.com/v1/embeddings").map_err(|_| {
            ApiError::OpenaiError(OpenaiError::from_http_status(
                500,
                "Could not parse URL".to_string(),
            ))
        })?;

        let res = client
            .post(url)
            .bearer_auth(self.openai_key.as_str())
            .json(&json!({
                "input": documents,
                "model": &self.model,
            }))
            .send()
            .await;

        match res {
            Ok(response) => {
                let data: Value = response.json().await.map_err(|e| {
                    log::error!("Could not parse response: {}", e);
                    ApiError::OpenaiError(OpenaiError::from_http_status(
                        500,
                        "Could not parse response".to_string(),
                    ))
                })?;
                let embeddings: Vec<Vec<f64>> = data["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| {
                        val["embedding"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|n| n.as_f64().unwrap())
                            .collect()
                    })
                    .collect();
                Ok(embeddings)
            }
            Err(err) => Err(ApiError::OpenaiError(OpenaiError::from_http_status(
                err.status().unwrap().as_u16(),
                err.to_string(),
            ))),
        }
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f64>, ApiError> {
        let client = Client::new();
        let url = Url::parse("https://api.openai.com/v1/embeddings").map_err(|_| {
            ApiError::OpenaiError(OpenaiError::from_http_status(
                500,
                "Could not parse URL".to_string(),
            ))
        })?;

        let res = client
            .post(url)
            .bearer_auth(&self.openai_key)
            .json(&json!({
                "input": text,
                "model": &self.model,
            }))
            .send()
            .await;

        match res {
            Ok(response) => {
                let data: Value = response.json().await.map_err(|e| {
                    log::error!("Could not parse response: {}", e);
                    ApiError::OpenaiError(OpenaiError::from_http_status(
                        500,
                        "Could not parse response".to_string(),
                    ))
                })?;
                let embedding: Vec<f64> = data["data"][0]["embedding"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_f64().unwrap())
                    .collect();
                Ok(embedding)
            }
            Err(err) => Err(ApiError::OpenaiError(OpenaiError::from_http_status(
                err.status().unwrap().as_u16(),
                err.to_string(),
            ))),
        }
    }
}
