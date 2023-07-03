//! API client

pub mod error;

use error::Error;
pub use newsie_api::http::auth::{LoginReqBody, LoginRespBody, NewUser, SignupRespBody};

/// API client
pub struct Client {
    /// Base URL
    pub url: String,
    /// Authentication token
    pub token: Option<String>,
}

impl Client {
    /// Creates a new API client
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            token: None,
        }
    }

    /// Sets the authentication token
    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }
}

impl Client {
    /// Signup a new user
    pub async fn signup(&self, new_user: NewUser) -> Result<SignupRespBody, Error> {
        Ok(reqwest::Client::new()
            .post(&format!("{}/auth/signup", self.url))
            .json(&new_user)
            .send()
            .await?
            .json::<SignupRespBody>()
            .await?)
    }

    /// Login a user
    pub async fn login(&self, email: &str, password: &str) -> Result<LoginRespBody, Error> {
        let body = LoginReqBody {
            email: email.to_string(),
            password: password.to_string(),
        };
        Ok(reqwest::Client::new()
            .post(&format!("{}/auth/login", self.url))
            .json(&body)
            .send()
            .await?
            .json::<LoginRespBody>()
            .await?)
    }

    /// Gets the user info
    pub async fn get_user_info(&self, email: &str, password: &str) -> Result<LoginRespBody, Error> {
        let body = LoginReqBody {
            email: email.to_string(),
            password: password.to_string(),
        };
        Ok(reqwest::Client::new()
            .get(&format!("{}/auth/me", self.url))
            .json(&body)
            .send()
            .await?
            .json::<LoginRespBody>()
            .await?)
    }
}
