//! Middlewares

use crate::svc::{self, Context};

use super::Request;

/// Middleware error
#[derive(Debug, thiserror::Error)]
pub enum MdwError {
    /// Bad request
    #[error("Bad request: {message}")]
    BadRequest {
        /// Message
        message: String,
    },
    /// Internal server error
    #[error("Server error")]
    Internal {
        /// Message
        message: String,
    },
    /// Invalid JWT token
    #[error("Invalid token: {message}")]
    InvalidToken {
        /// Message
        message: String,
    },
}

/// Extracts the authenticated user, and populated the [Context]
///
/// This is based on the "Bearer" token in the header.
pub async fn extract_user(ctx: &mut Context, req: &Request) -> Result<(), MdwError> {
    // Extract the auth token from the header
    let token = match req.headers().get(hyper::header::AUTHORIZATION) {
        Some(v) => match v.to_str() {
            Ok(s) => match s.strip_prefix("Bearer") {
                Some(s) => s,
                None => {
                    return Err(MdwError::BadRequest {
                        message: "Invalid authorization header".to_string(),
                    })
                }
            },
            Err(err) => {
                return Err(MdwError::BadRequest {
                    message: format!("{err}"),
                })
            }
        },
        None => return Ok(()),
    };

    // Query the service
    let user = match svc::auth::query_user_with_jwt(ctx, token).await {
        Ok(user) => user,
        Err(err) => match err {
            svc::auth::AuthError::InvalidToken { message } => {
                return Err(MdwError::InvalidToken { message })
            }
            svc::auth::AuthError::Internal { message } => {
                return Err(MdwError::Internal { message })
            }
        },
    };

    ctx.user = user;
    Ok(())
}
