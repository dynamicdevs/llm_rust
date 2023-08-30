use self::{aws_errors::AWSError, openai_errors::OpenaiError};

pub mod aws_errors;
pub mod openai_errors;
pub mod prompt_errors;
pub use prompt_errors::PromptError;

#[derive(Debug)]
pub enum ApiError {
    OpenaiError(OpenaiError),
    AWSError(AWSError),
    PromptError(PromptError),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::OpenaiError(err) => write!(f, "OpenAI error: {}", err),
            ApiError::AWSError(err) => write!(f, "AWS error: {}", err),
            ApiError::PromptError(err) => write!(f, "Prompt error: {}", err),
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApiError::OpenaiError(err) => Some(err),
            ApiError::AWSError(err) => Some(err),
            ApiError::PromptError(err) => Some(err),
        }
    }
}

impl std::error::Error for PromptError {}
