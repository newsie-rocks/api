//! Auth services

use argon2::{password_hash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::postgres::PostgresDb,
    error::Error,
    mdl::{NewUser, User, UserUpdateFields},
};

/// Authentication service
#[derive(Debug, Clone)]
pub struct AuthService {
    /// Postgres db
    pub db: PostgresDb,
    /// Secret used to sign the JWT token
    pub secret: String,
}

impl AuthService {
    /// Creates a new service instance
    pub fn new(postgres_pool: deadpool_postgres::Pool, secret: String) -> Self {
        let db = PostgresDb::new(postgres_pool);
        Self { db, secret }
    }
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
    pub async fn create(&self, mut new_user: NewUser) -> Result<User, Error> {
        // check that the user with the email exists
        if let Some(u) = self
            .db
            .read_user_with_email(&new_user.email)
            .await
            .map_err(Error::from)?
        {
            return Err(Error::InvalidRequest(
                format!("user with email '{email}' already exists", email = u.email),
                None,
            ));
        };

        // Hash the password
        let hashed_pwd = hash_password(&new_user.password)?;
        new_user.password = hashed_pwd;

        self.db.create_user(new_user).await
    }

    /// Queries a user with its ID
    pub async fn read(&self, user_id: Uuid) -> Result<Option<User>, Error> {
        self.db.read_user(user_id).await
    }

    /// Updates a user
    pub async fn update(&self, user_id: Uuid, mut fields: UserUpdateFields) -> Result<User, Error> {
        // Hash the password before updating it
        if let Some(password) = fields.password.as_ref() {
            let hashed_pwd = hash_password(password)?;
            fields.password = Some(hashed_pwd);
        }

        self.db.update_user(user_id, fields).await?;
        self.db
            .read_user(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("no user".to_string(), None))
    }

    /// Deletes a user
    pub async fn delete(&self, user_id: Uuid) -> Result<(), Error> {
        self.db.delete_user(user_id).await
    }

    /// Login a new user
    pub async fn login(&self, email: &str, password: &str) -> Result<User, Error> {
        let user = self.db.read_user_with_email(email).await?;
        if user.is_none() {
            return Err(Error::NotFound(
                format!("no user for email '{email}'"),
                None,
            ));
        }
        let user = user.unwrap();

        if !verify_password(&user.password, password)? {
            return Err(Error::Unauthenticated(
                format!("invalid password for email '{email}'"),
                None,
            ));
        }

        Ok(user)
    }

    /// Issues a JWT token for a user
    pub fn issue_token(&self, user: &User) -> Result<String, Error> {
        // define the token expiry
        let exp = time::OffsetDateTime::now_utc() + time::Duration::days(30);

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
    pub async fn read_with_token(&self, token: &str) -> Result<Option<User>, Error> {
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
fn hash_password(password: &str) -> Result<String, Error> {
    let salt = password_hash::SaltString::generate(&mut password_hash::rand_core::OsRng);
    let argon2 = argon2::Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verifies a hashed password
pub fn verify_password(hash: &str, password: &str) -> Result<bool, Error> {
    let parsed_hash = password_hash::PasswordHash::new(hash)?;
    let ok = argon2::Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;

    use super::*;

    use fake::{
        faker::{internet::en::FreeEmail, name::en::Name},
        Fake,
    };

    /// Setup a test
    async fn setup() -> (AuthService, User) {
        let cfg = AppConfig::load();
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
        (service, user)
    }

    /// Teardown
    async fn teardown(service: AuthService, user: User) {
        service.delete(user.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_user() {
        let (service, user) = setup().await;
        let updated_user = service
            .update(
                user.id,
                UserUpdateFields {
                    name: Some("__test__update".to_string()),
                    email: None,
                    password: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(updated_user.name, "__test__update".to_string());
        teardown(service, user).await;
    }

    #[tokio::test]
    async fn test_issue_token() {
        let (service, user) = setup().await;
        let _token = service.issue_token(&user).unwrap();
        teardown(service, user).await;
    }
}
