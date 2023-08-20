use core::fmt;

#[derive(Debug)]
pub enum PromptError {
    RenderError(String),
    DataNotProvided(String),
}

impl fmt::Display for PromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptError::RenderError(err) => write!(f, "Render Error: {}", err),
            PromptError::DataNotProvided(err) => write!(f, "Data Not Provided: {}", err),
        }
    }
}
