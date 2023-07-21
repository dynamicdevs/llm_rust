use std::time::Duration;
use tokio::time::sleep;

use rusoto_textract::{
    Block, DocumentLocation, GetDocumentTextDetectionRequest, S3Object,
    StartDocumentTextDetectionRequest, Textract, TextractClient,
};

use crate::errors::{aws_errors::AWSError, ApiError};

pub struct TextractService {
    pub client: TextractClient,
}
impl TextractService {
    pub fn new(client: TextractClient) -> Self {
        Self { client }
    }

    fn parse_s3_uri(&self, uri: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = uri.splitn(4, "/").collect();
        if parts.len() < 4 {
            return None;
        }
        Some((parts[2].to_string(), parts[3..].join("/")))
    }

    pub async fn pdf_to_text(&self, bucket_uri: &str) -> Result<Vec<String>, ApiError> {
        let (bucket, key) = self.parse_s3_uri(bucket_uri).ok_or(ApiError::AWSError(
            AWSError::new_malformed_uri(String::from("Malformed Bucket URI")),
        ))?;
        let request = StartDocumentTextDetectionRequest {
            document_location: DocumentLocation {
                s3_object: Some(S3Object {
                    bucket: Some(bucket.clone()),
                    name: Some(key.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let start_response = self
            .client
            .start_document_text_detection(request)
            .await
            .map_err(|e| {
                ApiError::AWSError(AWSError::new_server_error(String::from(e.to_string())))
            })?;
        let job_id = start_response.job_id.unwrap();

        let mut status = "IN_PROGRESS".to_string();
        while status == "IN_PROGRESS" {
            let mut blocks: Vec<Block> = vec![];
            let mut next_token: Option<String> = None;

            loop {
                let result_request = GetDocumentTextDetectionRequest {
                    job_id: job_id.clone(),
                    next_token: next_token.clone(),
                    ..Default::default()
                };

                let result_response = self
                    .client
                    .get_document_text_detection(result_request)
                    .await
                    .map_err(|e| {
                        ApiError::AWSError(AWSError::new_server_error(String::from(e.to_string())))
                    })?;

                status = result_response.job_status.unwrap();
                blocks.extend(result_response.blocks.unwrap_or_else(Vec::new));
                next_token = result_response.next_token;

                if status == "SUCCEEDED" {
                    if next_token.is_none() {
                        let text_blocks: Vec<String> = blocks
                            .iter()
                            .filter(|block| block.block_type.as_deref() == Some("WORD"))
                            .filter_map(|block| block.text.clone())
                            .collect();
                        return Ok(text_blocks);
                    }
                } else if next_token.is_none() {
                    break;
                }
            }

            sleep(Duration::from_secs(5)).await;
        }

        Err(ApiError::AWSError(AWSError::new_server_error(
            "Could not get text from PDF".to_string(),
        )))
    }
}
impl Default for TextractService {
    fn default() -> Self {
        Self {
            client: TextractClient::new(Default::default()),
        }
    }
}
