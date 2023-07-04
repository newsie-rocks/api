//! Auth services

use argon2::{password_hash, PasswordHasher, PasswordVerifier};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::{error::StoreError, user::UserStore};

/// Authentication service
#[derive(Debug, Clone)]
pub struct AuthService {
    /// User store
    pub store: UserStore,
    /// Secret used to sign the JWT token
    pub secret: String,
}

impl AuthService {
    /// Creates a new service instance
    pub fn new(postgres_pool: deadpool_postgres::Pool, secret: String) -> Self {
        let store = UserStore::new(postgres_pool);
        Self { store, secret }
    }
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
    /// Invalid credentials
    #[error("Invalid credentials: {message}")]
    InvalidCredentials {
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

impl From<StoreError> for AuthError {
    fn from(value: StoreError) -> Self {
        match value {
            StoreError::Internal { message } => AuthError::Internal { message },
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

/// User
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewUser {
    /// Name
    pub name: String,
    /// Email
    pub email: String,
    /// Password
    pub password: String,
}

/// User update
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserFields {
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

impl AuthService {
    /// Creates a new [User]
    pub async fn create(&self, mut new_user: NewUser) -> Result<User, AuthError> {
        // check that the user with the email exists
        if let Some(u) = self
            .store
            .read_with_email(&new_user.email)
            .await
            .map_err(AuthError::from)?
        {
            return Err(AuthError::Unauthorized {
                message: format!("user with email '{email}' already exists", email = u.email),
            });
        };

        // Hash the password
        let hashed_pwd = hash_password(&new_user.password)?;
        new_user.password = hashed_pwd;

        Ok(self.store.create(new_user).await?)
    }

    /// Queries a user with its ID
    pub async fn read(&self, user_id: Uuid) -> Result<Option<User>, AuthError> {
        Ok(self.store.read(user_id).await?)
    }

    /// Updates a user
    pub async fn update(&self, user_id: Uuid, mut fields: UserFields) -> Result<User, AuthError> {
        // Hash the password before updating it
        if let Some(password) = fields.password.as_ref() {
            let hashed_pwd = hash_password(password)?;
            fields.password = Some(hashed_pwd);
        }

        self.store.update(user_id, fields).await?;
        self.store
            .read(user_id)
            .await?
            .ok_or_else(|| AuthError::UserNotFound {
                message: "no user".to_string(),
            })
    }

    /// Deletes a user
    pub async fn delete(&self, user_id: Uuid) -> Result<(), AuthError> {
        Ok(self.store.delete(user_id).await?)
    }

    /// Login a new user
    pub async fn login(&self, email: &str, password: &str) -> Result<User, AuthError> {
        let user = self.store.read_with_email(email).await?;
        if user.is_none() {
            return Err(AuthError::UserNotFound {
                message: format!("no user for email '{email}'"),
            });
        }
        let user = user.unwrap();

        if !verify_password(&user.password, password)? {
            return Err(AuthError::InvalidCredentials {
                message: format!("invalid password for email '{email}'"),
            });
        }

        Ok(user)
    }

    /// Issues a JWT token for a user
    pub fn issue_token(&self, user: &User) -> Result<String, AuthError> {
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
            &jsonwebtoken::EncodingKey::from_secret(self.secret.as_bytes()),
        ) {
            Ok(t) => Ok(t),
            Err(err) => Err(err.into()),
        }
    }

    /// Queries a user with a JWT token
    pub async fn read_with_token(&self, token: &str) -> Result<Option<User>, AuthError> {
        // Decode the token
        let claims = match jsonwebtoken::decode::<AuthJwtClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        ) {
            Ok(data) => data.claims,
            Err(err) => {
                return Err(err.into());
            }
        };

        // Query the user by ID
        self.read(claims.user_id).await
    }
}

/// Hashes a password
fn hash_password(password: &str) -> Result<String, AuthError> {
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
    use std::future::Future;

    use super::*;

    use fake::{
        faker::{internet::en::FreeEmail, name::en::Name},
        Fake,
    };

    use crate::{
        config::AppConfig,
        svc::auth::{NewUser, UserFields},
    };

    // Test runner to setup and cleanup a test
    async fn run_test<F>(f: impl Fn(AuthService, User) -> F)
    where
        F: Future<Output = (AuthService, User)>,
    {
        let cfg = AppConfig::load().await;
        let service = AuthService::new(cfg.postgres.new_pool(), cfg.auth.secret.clone());

        // create dummy user
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let user = service
            .create(NewUser {
                name,
                email,
                password: "dummy".to_string(),
            })
            .await
            .unwrap();

        // Run the test
        let (service, user) = f(service, user).await;

        // cleanup
        service.delete(user.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_user() {
        run_test(|service, user| async {
            let updated_user = service
                .update(
                    user.id,
                    UserFields {
                        name: Some("__test__update".to_string()),
                        email: None,
                        password: None,
                    },
                )
                .await
                .unwrap();
            assert_eq!(updated_user.name, "__test__update".to_string());
            (service, user)
        })
        .await;
    }

    #[tokio::test]
    async fn test_issue_token() {
        run_test(|service, user| async {
            let _token = service.issue_token(&user).unwrap();
            (service, user)
        })
        .await;
    }
}
