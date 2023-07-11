//! Service

use anyhow::Error;
use newsie_client::{Client as ApiClient, NewUser, User};

use crate::svc::config::Config;

use self::db::DbClient;

mod config;
mod db;

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
}
