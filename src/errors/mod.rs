use self::openai_errors::OpenaiError;

pub mod openai_errors;

#[derive(Debug)]
pub enum ApiError {
    OpenaiError(OpenaiError),
}
