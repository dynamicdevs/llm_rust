#[derive(Debug, Clone)]
pub enum OpenaiError {
    InvalidAuthentication { code: u16, detail: String },
    IncorrectApiKey { code: u16, detail: String },
    NoOrganizationMembership { code: u16, detail: String },
    RateLimitExceeded { code: u16, detail: String },
    QuotaExceeded { code: u16, detail: String },
    ServerError { code: u16, detail: String },
    EngineOverloaded { code: u16, detail: String },
    UnknownError { code: u16, detail: String },
    GenericError(String),
}

impl OpenaiError {
    pub fn new_generic_error(msg: String) -> Self {
        OpenaiError::GenericError(msg)
    }

    pub fn from_http_status(code: u16, detail: String) -> Self {
        match code {
            401 => {
                // Choose appropriate error type based on detail
                if detail.contains("Incorrect API key") {
                    OpenaiError::IncorrectApiKey { code, detail }
                } else if detail.contains("You must be a member of an organization") {
                    OpenaiError::NoOrganizationMembership { code, detail }
                } else {
                    OpenaiError::InvalidAuthentication { code, detail }
                }
            }
            429 => {
                // Choose appropriate error type based on detail
                if detail.contains("exceeded your current quota") {
                    OpenaiError::QuotaExceeded { code, detail }
                } else {
                    OpenaiError::RateLimitExceeded { code, detail }
                }
            }
            500 => OpenaiError::ServerError { code, detail },
            503 => OpenaiError::EngineOverloaded { code, detail },
            _ => OpenaiError::UnknownError { code, detail }, // For other error codes not explicitly handled
        }
    }
}

impl std::fmt::Display for OpenaiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenaiError::InvalidAuthentication { code, detail } => {
                write!(
                    f,
                    "Error  code {}: Invalid Authentication - {}",
                    code, detail
                )
            }
            OpenaiError::IncorrectApiKey { code, detail } => {
                write!(
                    f,
                    "Error  code {}: Incorrect API key provided - {}",
                    code, detail
                )
            }
            OpenaiError::NoOrganizationMembership { code, detail } => {
                write!(
                    f,
                    "Error code {}: You must be a member of an organization to use the API - {}",
                    code, detail
                )
            }
            OpenaiError::RateLimitExceeded { code, detail } => {
                write!(
                    f,
                    "Error code {}: Rate limit reached for requests - {}",
                    code, detail
                )
            }
            OpenaiError::QuotaExceeded { code, detail } => {
                write!(f, "Error code {}: You exceeded your current quota, please check your plan and billing details - {}", code, detail)
            }
            OpenaiError::ServerError { code, detail } => {
                write!(
                    f,
                    "Error code {}: The server had an error while processing your request - {}",
                    code, detail
                )
            }
            OpenaiError::EngineOverloaded { code, detail } => {
                write!(f, "Error code {}: The engine is currently overloaded, please try again later - {}", code, detail)
            }
            OpenaiError::UnknownError { code, detail } => {
                write!(f, "Error code {}: Unknown error - {}", code, detail)
            }
            OpenaiError::GenericError(_) => {
                write!(f, "An unknown error occurred with the OpenAI API.")
            }
        }
    }
}

impl std::error::Error for OpenaiError {}
