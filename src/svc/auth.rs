//! Auth services

use serde::{Deserialize, Serialize};

use super::Context;

/// User
#[derive(Debug)]
pub struct User {
    /// User ID
    pub id: i32,
    /// User Name
    pub name: String,
}

/// New user
#[derive(Debug)]
pub struct NewUser {
    /// User Name
    pub name: String,
}

/// User update
#[derive(Debug)]
pub struct UserFields {
    /// User ID
    pub id: i32,
    /// User Name
    pub name: Option<String>,
}

/// Authentication JWT
#[derive(Debug, Serialize, Deserialize)]
struct AuthJwtClaims {
    /// Subject
    sub: String,
    /// Expiry
    exp: usize,
    /// User ID
    user_id: i32,
}

/// Authentication service error
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Invalid JWT token
    #[error("Invalid JWT token: {message}")]
    InvalidToken {
        /// Message
        message: String,
    },
    /// Internal error
    #[error("{message}")]
    Internal {
        /// Message
        message: String,
    },
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidSignature => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_) => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::RsaFailedSigning => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithmName => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidKeyFormat => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(_) => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidAudience => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidSubject => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::ImmatureSignature => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::MissingAlgorithm => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::Base64(_) => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::Json(_) => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::Utf8(_) => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            jsonwebtoken::errors::ErrorKind::Crypto(_) => AuthError::InvalidToken {
                message: "invalid token".to_string(),
            },
            _ => todo!("Missing JWT error handling"),
        }
    }
}

impl From<crate::data::DbError> for AuthError {
    fn from(value: crate::data::DbError) -> Self {
        match value {
            crate::data::DbError::Internal { message } => AuthError::Internal { message },
        }
    }
}

/// Queries a user with a JWT token
pub async fn query_user_with_jwt(ctx: &Context, token: &str) -> Result<Option<User>, AuthError> {
    // Decode the token
    let claims = match jsonwebtoken::decode::<AuthJwtClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(ctx.auth_secret.as_ref()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(err) => return Err(err.into()),
    };

    // Query the user by ID
    query_user_with_id(ctx, claims.user_id).await
}

/// Issues a JWT token for a user
pub fn issue_user_token(ctx: &Context, user: &User) -> Result<String, AuthError> {
    let claims = AuthJwtClaims {
        sub: "auth".to_string(),
        exp: 3600,
        user_id: user.id,
    };

    match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(ctx.auth_secret.as_ref()),
    ) {
        Ok(t) => Ok(t),
        Err(err) => Err(err.into()),
    }
}

/// Queries a user
pub async fn query_user_with_id(ctx: &Context, user_id: i32) -> Result<Option<User>, AuthError> {
    crate::data::user::read(ctx, user_id)
        .await
        .map(|user| {
            user.map(|u| User {
                id: u.id,
                name: u.name,
            })
        })
        .map_err(|err| err.into())
}

/// Signup a new user
pub fn create_user(ctx: &Context) -> Result<(), ()> {
    todo!("Signup a new user")
}
