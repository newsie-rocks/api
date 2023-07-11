//! Error

use salvo::prelude::*;
use serde::{Deserialize, Serialize};

/// Error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// InvalidRequest
    #[error("error: {0}")]
    InvalidRequest(String, Option<String>),
    /// Resource not found
    #[error("error: {0}")]
    NotFound(String, Option<String>),
    /// Unauthenticated
    #[error("error: {0}")]
    Unauthenticated(String, Option<String>),
    /// Internal server or service error
    #[error("error: {0}")]
    Internal(String, Option<String>),
}

impl Error {
    /// Returns the main message
    pub fn message(&self) -> String {
        match self {
            Error::InvalidRequest(msg, _) => msg.clone(),
            Error::NotFound(msg, _) => msg.clone(),
            Error::Unauthenticated(msg, _) => msg.clone(),
            Error::Internal(msg, _) => msg.clone(),
        }
    }

    /// Returns the error code
    pub fn code(&self) -> String {
        match self {
            Error::InvalidRequest(_, _) => "INVALID_REQUEST".to_string(),
            Error::NotFound(_, _) => "NOT_FOUND".to_string(),
            Error::Unauthenticated(_, _) => "NOT_AUTHENTICATED".to_string(),
            Error::Internal(_, _) => "INTERNAL".to_string(),
        }
    }

    /// Returns the HTTP code
    pub fn http_code(&self) -> StatusCode {
        match self {
            Error::InvalidRequest(_, _) => StatusCode::BAD_REQUEST,
            Error::NotFound(_, _) => StatusCode::NOT_FOUND,
            Error::Unauthenticated(_, _) => StatusCode::UNAUTHORIZED,
            Error::Internal(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<deadpool_postgres::PoolError> for Error {
    fn from(value: deadpool_postgres::PoolError) -> Self {
        Error::Internal(value.to_string(), None)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(value: tokio_postgres::Error) -> Self {
        Error::Internal(value.to_string(), None)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Error::Unauthenticated(format!("invalid token ({value})"), None)
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Error::Internal(format!("{value}"), None)
    }
}

impl From<salvo::http::ParseError> for Error {
    fn from(value: salvo::http::ParseError) -> Self {
        Error::InvalidRequest(value.to_string(), None)
    }
}

/// Http error response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HttpErrorResponse {
    /// Main error
    pub error: HttpError,
}

/// Error JSON shape
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HttpError {
    /// Code (string)
    pub code: String,
    /// Message
    pub message: String,
    /// Other details
    pub detail: Option<String>,
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let code = self.code();
        let http_code = self.http_code();
        let (message, detail) = match self {
            Error::InvalidRequest(message, detail) => (message, detail),
            Error::NotFound(message, detail) => (message, detail),
            Error::Unauthenticated(message, detail) => (message, detail),
            Error::Internal(message, detail) => (message, detail),
        };

        let err = HttpErrorResponse {
            error: HttpError {
                code,
                message,
                detail,
            },
        };
        res.status_code(http_code);
        res.render(Json(err));
    }
}

// NB: needed for OpenAPI specs
impl EndpointOutRegister for Error {
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        let schema = HttpErrorResponse::to_schema(components);
        let content = salvo::oapi::Content::new(schema);

        let res = salvo::oapi::Response::new("Bad request")
            .add_content("application/json", content.clone());
        operation.responses.insert("400", res);

        let res = salvo::oapi::Response::new("Unauthorized")
            .add_content("application/json", content.clone());
        operation.responses.insert("401", res);

        let res =
            salvo::oapi::Response::new("Server error").add_content("application/json", content);
        operation.responses.insert("500", res);
    }
}

impl From<async_openai::error::OpenAIError> for Error {
    fn from(value: async_openai::error::OpenAIError) -> Self {
        Error::Internal(format!("OpenAI error ({value})"), None)
    }
}
