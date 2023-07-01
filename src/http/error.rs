//! HTTP error handling

use salvo::prelude::*;
use serde::Serialize;

use crate::svc;

/// HTTP API error
#[derive(Debug, thiserror::Error, Serialize)]

pub enum HttpError {
    /// Invalid client request
    #[error("INVALID_REQUEST:{0}")]
    BadRequest(String, Option<String>),
    /// Unauthorized
    #[error("UNAUTHORIZED:{0}")]
    Unauthorized(String, Option<String>),
    /// Internal server error
    #[error("INTERNAL:{0}")]
    Internal(String, Option<String>),
}

impl HttpError {
    /// Returns the error code
    fn code(&self) -> String {
        match self {
            HttpError::BadRequest(_, _) => "INVALID_REQUEST".into(),
            HttpError::Unauthorized(_, _) => "UNAUTHORIZED".into(),
            HttpError::Internal(_, _) => "INTERNAL".into(),
        }
    }

    /// Returns the status code
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::BadRequest(_, _) => StatusCode::BAD_REQUEST,
            HttpError::Unauthorized(_, _) => StatusCode::UNAUTHORIZED,
            HttpError::Internal(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Error response
#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    /// Main error
    error: ErrorShape,
}

/// Error JSON shape
#[derive(Debug, Serialize, ToSchema)]
struct ErrorShape {
    /// Code (string)
    code: String,
    /// Message
    message: String,
    /// Other details
    detail: Option<String>,
}

#[async_trait]
impl Writer for HttpError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let code = self.code();
        let status_code = self.status_code();
        let error = match self {
            HttpError::BadRequest(message, detail) => ErrorShape {
                code,
                message,
                detail,
            },
            HttpError::Unauthorized(message, detail) => ErrorShape {
                code,
                message,
                detail,
            },
            HttpError::Internal(message, detail) => ErrorShape {
                code,
                message,
                detail,
            },
        };
        res.status_code(status_code);
        res.render(Json(ErrorResponse { error }));
    }
}

// NB: needed for OpenAPI specs
impl EndpointOutRegister for HttpError {
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        let schema = ErrorResponse::to_schema(components);
        let content = salvo::oapi::Content::new(schema);

        let res = salvo::oapi::Response::new("Bad request")
            .add_content("application/json", content.clone());
        operation.responses.insert("400", res);

        let res = salvo::oapi::Response::new("Unauthorized")
            .add_content("application/json", content.clone());
        operation.responses.insert("401", res);

        let res = salvo::oapi::Response::new("Server error")
            .add_content("application/json", content.clone());
        operation.responses.insert("500", res);
    }
}

impl From<salvo::http::ParseError> for HttpError {
    fn from(value: salvo::http::ParseError) -> Self {
        HttpError::BadRequest(value.to_string(), None)
    }
}

impl From<svc::auth::AuthError> for HttpError {
    fn from(value: svc::auth::AuthError) -> Self {
        match value {
            svc::auth::AuthError::InvalidToken { message } => {
                HttpError::Unauthorized(message, None)
            }
            svc::auth::AuthError::UserNotFound { message } => {
                HttpError::Unauthorized(message, None)
            }
            svc::auth::AuthError::Unauthorized { message } => {
                HttpError::Unauthorized(message, None)
            }
            svc::auth::AuthError::Internal { message } => HttpError::Internal(message, None),
        }
    }
}
