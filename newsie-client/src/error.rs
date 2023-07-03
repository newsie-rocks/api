//! Error

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    /// Error message
    message: String,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}
