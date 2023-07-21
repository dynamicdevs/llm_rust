use self::{aws_errors::AWSError, openai_errors::OpenaiError};

pub mod aws_errors;
pub mod openai_errors;

#[derive(Debug)]
pub enum ApiError {
    OpenaiError(OpenaiError),
    AWSError(AWSError),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::OpenaiError(err) => write!(f, "OpenAI error: {}", err),
            ApiError::AWSError(err) => write!(f, "AWS error: {}", err),
        }
    }
}
