//! Middlewares

use crate::svc::{self, Context};

use super::HttpRequest;

/// Middleware error
#[derive(Debug, thiserror::Error)]
pub enum MdwError {
    /// Bad request
    #[error("Bad request: {message}")]
    InvalidReq {
        /// Message
        message: String,
    },
    /// Unauthorized
    #[error("Unauthorized: {message}")]
    Unauthorized {
        /// Message
        message: String,
    },
    /// Internal server error
    #[error("Server error")]
    Internal {
        /// Message
        message: String,
    },
}

impl From<svc::auth::AuthError> for MdwError {
    fn from(value: svc::auth::AuthError) -> Self {
        match value {
            svc::auth::AuthError::InvalidToken { message } => Self::Unauthorized { message },
            svc::auth::AuthError::UserNotFound { message } => Self::Unauthorized { message },
            svc::auth::AuthError::Unauthorized { message } => Self::Unauthorized { message },
            svc::auth::AuthError::Internal { message } => Self::Internal { message },
        }
    }
}

/// Extracts the authenticated user, and populated the [Context]
///
/// This is based on the "Bearer" token in the header.
pub async fn extract_user(ctx: &mut Context, req: &HttpRequest) -> Result<(), MdwError> {
    // Extract the auth token from the AUTHORIZATION header
    let mut token = match req.headers().get(hyper::header::AUTHORIZATION) {
        Some(v) => match v.to_str() {
            Ok(s) => match s.strip_prefix("Bearer") {
                Some(s) => Some(s.to_string()),
                None => {
                    return Err(MdwError::InvalidReq {
                        message: "Invalid authorization header".to_string(),
                    })
                }
            },
            Err(err) => {
                return Err(MdwError::InvalidReq {
                    message: format!("{err}"),
                })
            }
        },
        None => None,
    };

    // If undefined, try extracting the token from the cookie
    if token.is_none() {
        match super::auth::parse_auth_cookie(req) {
            Ok(cookie_auth_token) => token = cookie_auth_token,
            Err(err) => {
                return Err(MdwError::InvalidReq {
                    message: format!("{err}"),
                })
            }
        };
    }

    // Return if non token
    if let Some(t) = token {
        let user = svc::auth::read_user_with_token(ctx, t.as_str()).await?;
        ctx.user = user;
    }
    Ok(())
}
