//! API client

pub mod error;

use error::Error;
use newsie_api::error::HttpErrorResponse;
pub use newsie_api::{
    http::{
        auth::{GetUserRespBody, LoginReqBody, LoginRespBody, SignupRespBody},
        feed::GetFeedsRespBody,
        summary::SummariesRespBody,
    },
    mdl::{Feed, FeedUpdate, NewUser, Subscription, SubscriptionUpdate, Summary, User, UserUpdate},
};
use reqwest::header::{HeaderMap, AUTHORIZATION};

// Re-exports

/// API client
#[derive(Debug, Clone)]
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
    pub fn token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    /// Removes the authentication token
    pub fn unset_token(&mut self) -> &mut Self {
        self.token = None;
        self
    }
}

impl Client {
    /// Signup a new user
    pub async fn signup(&mut self, new_user: NewUser) -> Result<SignupRespBody, Error> {
        let mut headers = HeaderMap::new();

        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .post(&format!("{}/auth/signup", self.url))
            .headers(headers)
            .json(&new_user)
            .send()
            .await?;

        if res.status().is_success() {
            let ok = res.json::<SignupRespBody>().await?;
            self.token = Some(ok.token.clone());
            Ok(ok)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }

    /// Login a user
    pub async fn login(&mut self, email: &str, password: &str) -> Result<LoginRespBody, Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let body = LoginReqBody {
            email: email.to_string(),
            password: password.to_string(),
        };

        let res = reqwest::Client::new()
            .post(&format!("{}/auth/login", self.url))
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if res.status().is_success() {
            let ok = res.json::<LoginRespBody>().await?;
            self.token = Some(ok.token.clone());
            Ok(ok)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }

    /// Gets the user info
    pub async fn me(&self) -> Result<GetUserRespBody, Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .get(&format!("{}/auth/me", self.url))
            .headers(headers)
            .send()
            .await?;

        if res.status().is_success() {
            let ok = res.json::<GetUserRespBody>().await?;
            Ok(ok)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }

    /// Update the user
    pub async fn update_me(&self, fields: UserUpdate) -> Result<User, Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .patch(&format!("{}/auth/me", self.url))
            .headers(headers)
            .json(&fields)
            .send()
            .await?;

        if res.status().is_success() {
            let ok = res.json::<GetUserRespBody>().await?;
            Ok(ok.user)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }

    /// Deletes the user
    pub async fn delete_me(&mut self) -> Result<(), Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .delete(&format!("{}/auth/me", self.url))
            .headers(headers)
            .send()
            .await?;

        if res.status().is_success() {
            self.unset_token();
            Ok(())
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }

    /// Update the user subscription
    pub async fn update_subscription(&self, update: SubscriptionUpdate) -> Result<User, Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .put(&format!("{}/auth/me/subscription", self.url))
            .headers(headers)
            .json(&update)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.json::<GetUserRespBody>().await?;
            Ok(body.user)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }
}

impl Client {
    /// Get the user feeds
    pub async fn get_feeds(&self) -> Result<Vec<Feed>, Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .get(&format!("{}/feeds", self.url))
            .headers(headers)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.json::<GetFeedsRespBody>().await?;
            Ok(body.feeds)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }

    /// Sync the user feeds
    pub async fn sync_feeds(&self, feeds: &[FeedUpdate]) -> Result<Vec<Feed>, Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .put(&format!("{}/feeds", self.url))
            .headers(headers)
            .json(feeds)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.json::<GetFeedsRespBody>().await?;
            Ok(body.feeds)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }
}

impl Client {
    /// Summarize a list of articles
    pub async fn summarize(&self, urls: &[&str]) -> Result<Vec<Summary>, Error> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        let res = reqwest::Client::new()
            .post(&format!("{}/summaries", self.url))
            .headers(headers)
            .json(&urls.iter().map(|url| url.to_string()).collect::<Vec<_>>())
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.json::<SummariesRespBody>().await?;
            Ok(body.summaries)
        } else {
            let err = res.json::<HttpErrorResponse>().await?;
            Err(err.into())
        }
    }
}
