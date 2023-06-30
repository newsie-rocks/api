//! Auth services

use argon2::{password_hash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Context;

/// User
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "docgen", derive(utoipa::ToSchema))]
pub struct User {
    /// ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Email
    pub email: String,
    /// Password
    #[serde(skip)]
    pub password: String,
}

/// New user
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "docgen", derive(utoipa::ToSchema))]
pub struct NewUser {
    /// Name
    pub name: String,
    /// Email
    pub email: String,
    /// Password
    pub password: String,
}

/// User update
#[derive(Debug)]
pub struct UserFields {
    /// ID
    pub id: Uuid,
    /// Name
    pub name: Option<String>,
    /// Email
    pub email: Option<String>,
    /// Password
    pub password: Option<String>,
}

/// Authentication JWT
#[derive(Debug, Serialize, Deserialize)]
struct AuthJwtClaims {
    /// Subject
    sub: String,
    /// Expiry
    exp: usize,
    /// User ID
    user_id: Uuid,
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
    /// User not found
    #[error("Invalid user: {message}")]
    UserNotFound {
        /// Message
        message: String,
    },
    /// Unauthorized
    #[error("Unauthorized")]
    Unauthorized {
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
        AuthError::InvalidToken {
            message: format!("invalid token: {value}"),
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

impl From<password_hash::Error> for AuthError {
    fn from(value: password_hash::Error) -> Self {
        AuthError::Internal {
            message: format!("{value}"),
        }
    }
}

/// Creates a new [User]
pub async fn create_user(ctx: &Context, mut new_user: NewUser) -> Result<User, AuthError> {
    // check that the user with the email exists
    if let Some(u) = crate::data::user::read_with_email(ctx, &new_user.email)
        .await
        .map_err(|err| AuthError::from(err))?
    {
        return Err(AuthError::Unauthorized {
            message: format!("user with email '{email}' already exists", email = u.email),
        });
    };

    // Hash the password
    let hashed_pwd = hash_password(&new_user.password)?;
    new_user.password = hashed_pwd;

    Ok(crate::data::user::create(ctx, new_user).await?)
}

/// Queries a user with its ID
pub async fn read_user_with_id(ctx: &Context, user_id: Uuid) -> Result<Option<User>, AuthError> {
    crate::data::user::read(ctx, user_id)
        .await
        .map_err(|err| err.into())
}

/// Updates a user
pub async fn update_user(ctx: &Context, mut fields: UserFields) -> Result<User, AuthError> {
    // Hash the password
    if let Some(password) = fields.password.as_ref() {
        let hashed_pwd = hash_password(password)?;
        fields.password = Some(hashed_pwd);
    }

    let user_id = fields.id;
    crate::data::user::update(ctx, fields).await?;
    crate::data::user::read(ctx, user_id)
        .await?
        .ok_or_else(|| AuthError::UserNotFound {
            message: "no user".to_string(),
        })
}

/// Deletes a user
pub async fn delete_user(ctx: &Context, user: User) -> Result<(), AuthError> {
    crate::data::user::delete(ctx, user)
        .await
        .map_err(|err| err.into())
}

/// Login a new user
pub async fn login(ctx: &Context, email: &str, password: &str) -> Result<User, AuthError> {
    let user = crate::data::user::read_with_email(ctx, email).await?;
    if user.is_none() {
        return Err(AuthError::UserNotFound {
            message: format!("no user for email '{email}'"),
        });
    }
    let user = user.unwrap();

    if !verify_password(&user.password, password)? {
        return Err(AuthError::UserNotFound {
            message: format!("no user for email '{email}'"),
        });
    }

    Ok(user)
}

/// Issues a JWT token for a user
pub fn issue_user_token(ctx: &Context, user: &User) -> Result<String, AuthError> {
    // define the token expiry
    let exp = time::OffsetDateTime::now_utc() + time::Duration::minutes(60);

    let claims = AuthJwtClaims {
        sub: "auth".to_string(),
        exp: exp.unix_timestamp().try_into().unwrap(),
        user_id: user.id,
    };

    match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(ctx.cfg.auth.secret.as_ref()),
    ) {
        Ok(t) => Ok(t),
        Err(err) => Err(err.into()),
    }
}

/// Queries a user with a JWT token
pub async fn read_user_with_token(ctx: &Context, token: &str) -> Result<Option<User>, AuthError> {
    // Decode the token
    let claims = match jsonwebtoken::decode::<AuthJwtClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(ctx.cfg.auth.secret.as_ref()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(err) => {
            return Err(err.into());
        }
    };

    // Query the user by ID
    read_user_with_id(ctx, claims.user_id).await
}

/// Hashes a password
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = password_hash::SaltString::generate(&mut password_hash::rand_core::OsRng);
    let argon2 = argon2::Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verifies a hashed password
pub fn verify_password(hash: &str, password: &str) -> Result<bool, AuthError> {
    let parsed_hash = password_hash::PasswordHash::new(hash)?;
    let ok = argon2::Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(ok)
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use crate::{
        config::AppConfig,
        svc::{
            auth::{NewUser, UserFields},
            Context,
        },
    };

    /// Initializes a dummy [Context] for tests
    async fn init_ctx() -> Context {
        let cfg = AppConfig::load().await;
        let db_pool = cfg.postgres.pool();

        Context {
            cfg,
            db_pool: Arc::new(db_pool),
            user: None,
        }
    }

    #[tokio::test]
    async fn create_user() {
        let ctx = init_ctx().await;

        let input = NewUser {
            name: "test_user".to_string(),
            email: "test@nicklabs.io".to_string(),
            password: "dummy".to_string(),
        };
        let user = super::create_user(&ctx, input).await.unwrap();

        assert_eq!(user.name, "test_user".to_string())
    }

    #[tokio::test]
    async fn update_user() {
        let ctx = init_ctx().await;

        let input = NewUser {
            name: "test_user".to_string(),
            email: "test@nicklabs.io".to_string(),
            password: "dummy".to_string(),
        };
        let user = super::create_user(&ctx, input).await.unwrap();

        let fields = UserFields {
            id: user.id,
            name: Some("albert".to_string()),
            email: None,
            password: None,
        };
        let updated_user = super::update_user(&ctx, fields).await.unwrap();
        assert_eq!(updated_user.name, "albert".to_string());
    }

    #[tokio::test]
    async fn delete_user() {
        let ctx = init_ctx().await;

        let input = NewUser {
            name: "test_user".to_string(),
            email: "test@nicklabs.io".to_string(),
            password: "dummy".to_string(),
        };
        let user = super::create_user(&ctx, input).await.unwrap();

        super::delete_user(&ctx, user).await.unwrap();
    }

    #[tokio::test]
    async fn issue_token() {
        let ctx: Context = init_ctx().await;

        let input = NewUser {
            name: "test_user".to_string(),
            email: "test@nicklabs.io".to_string(),
            password: "dummy".to_string(),
        };
        let user = super::create_user(&ctx, input).await.unwrap();

        let token = super::issue_user_token(&ctx, &user).unwrap();

        let db_user = super::read_user_with_token(&ctx, &token)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(db_user.id, user.id)
    }
}
