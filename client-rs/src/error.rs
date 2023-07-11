//! Error

use newsie_api::error::HttpErrorResponse;

#[derive(Debug, thiserror::Error)]
#[error("{code}: {message}")]
pub struct Error {
    /// Code
    code: String,
    /// Error message
    message: String,
}

impl From<HttpErrorResponse> for Error {
    fn from(value: HttpErrorResponse) -> Self {
        Error {
            code: value.error.code,
            message: value.error.message,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error {
            code: "INTERNAL".to_string(),
            message: value.to_string(),
        }
    }
}
