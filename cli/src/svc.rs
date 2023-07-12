//! Service

use anyhow::Error;
use newsie_client::{Client as ApiClient, NewUser, User};

use crate::{
    db::DbClient,
    model::{Config, Feed},
};

/// Service
pub struct Service {
    /// DB client
    db: DbClient,
    /// API client
    api: ApiClient,
}

impl Service {
    /// Instantiates a new Service
    pub fn new() -> Result<Self, Error> {
        // init DB client
        DbClient::init_db_file()?;
        let db_client = DbClient::new()?;
        if !db_client.is_db_schema_init()? {
            db_client.init_db_schema()?;
        }

        // read the config
        let config = Self::get_or_init_config(&db_client)?;

        // init API client
        let api_client = ApiClient::new(&config.api_url);

        Ok(Self {
            db: db_client,
            api: api_client,
        })
    }

    /// Gets or intitializes the default config
    fn get_or_init_config(client: &DbClient) -> Result<Config, Error> {
        if let Some(config) = client.read_config()? {
            Ok(config)
        } else {
            Ok(client.create_config(Config::default())?)
        }
    }
}

impl Service {
    /// Returns the config
    pub fn get_config(&self) -> Result<Config, Error> {
        Ok(self.db.read_config()?.unwrap())
    }

    /// Updates the config
    pub fn update_config(&self, config: Config) -> Result<Config, Error> {
        self.db.update_config(config)
    }

    /// Saves the token in the config
    pub fn save_token(&self, token: &str) -> Result<(), Error> {
        let mut config = self.db.read_config()?.unwrap();
        config.token = Some(token.to_string());
        self.db.update_config(config)?;
        Ok(())
    }
}

impl Service {
    /// Signups a new user
    pub async fn signup(&mut self, new_user: NewUser) -> Result<User, Error> {
        let res = self.api.signup(new_user).await?;
        let token = res.token;
        let user = res.user;
        self.save_token(&token)?;
        Ok(user)
    }

    /// Login a user
    pub async fn login(&mut self, email: &str, password: &str) -> Result<User, Error> {
        let res = self.api.login(email, password).await?;
        let token = res.token;
        let user = res.user;
        self.save_token(&token)?;
        Ok(user)
    }

    /// Returns the current user
    pub async fn me(&self) -> Result<User, Error> {
        let res = self.api.me().await?;
        Ok(res.user)
    }
}

impl Service {
    /// Returns the db feeds
    pub async fn get_feeds(&self) -> Result<Vec<Feed>, Error> {
        self.db.get_feeds().await
    }

    /// Adds a feed
    pub async fn add_feeds(&mut self, feeds: Vec<Feed>) -> Result<Vec<Feed>, Error> {
        self.db.create_feeds(feeds).await
    }

    /// Removes feeds
    pub async fn remove_feeds(&mut self, feeds_urls: Vec<String>) -> Result<(), Error> {
        self.db.remove_feeds(feeds_urls).await
    }

    /// Retrieves the feed articles
    pub async fn get_articles(&self, feed: &Feed) -> Result<Vec<Article>, Error> {
        let channel = feed.load().await?;
        let mut articles = vec![];
        for item in channel.items {
            articles.push(Article {
                feed_url: feed.url.clone(),
                url: item.link.ok_or(Error::msg("Missing article link"))?,
                title: item.title,
            });
        }
        Ok(articles)
    }
}
