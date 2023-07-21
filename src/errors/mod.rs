use self::{aws_errors::AWSError, openai_errors::OpenaiError};

pub mod aws_errors;
pub mod openai_errors;

#[derive(Debug)]
pub enum ApiError {
    OpenaiError(OpenaiError),
    AWSError(AWSError),
}
