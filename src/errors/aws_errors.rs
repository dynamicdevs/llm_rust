#[derive(Debug, Clone)]
pub enum AWSError {
    InvalidAuthentication { msg: String },
    ServerError { msg: String },
    GenericError { msg: String },
    MalformedUri { msg: String },
}

impl AWSError {
    pub fn new_malformed_uri(msg: String) -> Self {
        AWSError::MalformedUri { msg }
    }

    pub fn new_generic_error(msg: String) -> Self {
        AWSError::GenericError { msg }
    }

    pub fn new_invalid_authentication(msg: String) -> Self {
        AWSError::InvalidAuthentication { msg }
    }

    pub fn new_server_error(msg: String) -> Self {
        AWSError::ServerError { msg }
    }
}

impl std::fmt::Display for AWSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AWSError::InvalidAuthentication { msg } => {
                write!(f, "Error: Invalid Authentication - {}", msg)
            }
            AWSError::ServerError { msg } => write!(f, "Error: Server Error - {}", msg),

            AWSError::GenericError { msg } => write!(f, "Error: {}", msg),

            AWSError::MalformedUri { msg } => write!(f, "Error: Malformed URI - {}", msg),
        }
    }
}

impl std::error::Error for AWSError {}
