//! HTTP error handling

use hyper::{header::CONTENT_TYPE, Body, StatusCode};
use serde::Serialize;

use crate::svc;

use super::{mdw::MdwError, HttpResponse};

/// HTTP error kind
#[derive(Debug, Serialize)]
pub enum HttpErrorKind {
    /// Invalid client request
    InvalidRequest,
    /// Unauthoprized
    Unauthorized,
    /// Internal server error
    Internal,
}

/// HTTP API error
#[derive(Debug, thiserror::Error, Serialize)]
#[error("{kind:?}:{message:?}")]
pub struct HttpError {
    /// Type
    pub kind: HttpErrorKind,
    /// Message
    pub message: String,
    /// Details
    pub details: Option<String>,
}

impl HttpError {
    /// Instantiates a new [HttpError]
    pub fn new(kind: HttpErrorKind, message: String, details: Option<String>) -> Self {
        Self {
            kind,
            message,
            details,
        }
    }

    /// Returns the HTTP status code
    pub fn status(self) -> StatusCode {
        match &self.kind {
            HttpErrorKind::InvalidRequest => StatusCode::BAD_REQUEST,
            HttpErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            HttpErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Converts a [HttpError] into a HTTP response
    pub fn response(self) -> HttpResponse {
        let body_json = serde_json::to_string(&self).unwrap();
        let body = Body::from(body_json);

        hyper::Response::builder()
            .status(self.status())
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap()
    }
}

impl From<hyper::Error> for HttpError {
    fn from(value: hyper::Error) -> Self {
        HttpError::new(HttpErrorKind::Internal, format!("{value}"), None)
    }
}

impl From<self::MdwError> for HttpError {
    fn from(value: self::MdwError) -> Self {
        match value {
            self::MdwError::InvalidReq { message } => {
                HttpError::new(HttpErrorKind::InvalidRequest, message, None)
            }
            self::MdwError::Unauthorized { message } => {
                HttpError::new(HttpErrorKind::InvalidRequest, message, None)
            }
            self::MdwError::Internal { message } => {
                HttpError::new(HttpErrorKind::Internal, message, None)
            }
        }
    }
}

impl From<svc::auth::AuthError> for HttpError {
    fn from(value: svc::auth::AuthError) -> Self {
        match value {
            svc::auth::AuthError::InvalidToken { message } => {
                HttpError::new(HttpErrorKind::Unauthorized, message, None)
            }
            svc::auth::AuthError::UserNotFound { message } => {
                HttpError::new(HttpErrorKind::Unauthorized, message, None)
            }
            svc::auth::AuthError::Unauthorized { message } => {
                HttpError::new(HttpErrorKind::InvalidRequest, message, None)
            }
            svc::auth::AuthError::Internal { message } => {
                HttpError::new(HttpErrorKind::Internal, message, None)
            }
        }
    }
}
