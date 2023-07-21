use async_trait::async_trait;

use crate::errors::ApiError;

#[async_trait]
pub trait Embedder: Send {
    async fn embed_documents(&self, documents: Vec<String>) -> Result<Vec<Vec<f64>>, ApiError>;
    async fn embed_query(&self, text: &str) -> Result<Vec<f64>, ApiError>;
}
