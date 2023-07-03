//! Configuration

use std::{fs, path::PathBuf};

use anyhow::Result;
use inquire::Text;
use newsie_client::Client;
use serde::{Deserialize, Serialize};

use crate::util::{success, warn, ResultExt};

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API URL
    pub url: String,
    /// Authentication token
    pub token: Option<String>,
}

impl Config {
    /// Returns the default config path
    fn default_path() -> PathBuf {
        dirs::config_dir().unwrap().join("Newsie/config.toml")
    }

    /// Loads the configuration from the filesystem
    pub fn load() -> Result<Option<Self>> {
        let path = Self::default_path();
        if path.exists() {
            let cfg_str = fs::read_to_string(&path)?;
            Ok(Some(toml::from_str::<Config>(&cfg_str)?))
        } else {
            Ok(None)
        }
    }

    /// Saves the config to the filesystem
    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::default_path();
        // NB : we need to create the parent directory
        fs::create_dir_all(path.parent().unwrap())?;
        let cfg_str = toml::to_string(self)?;
        fs::write(&path, cfg_str)?;
        Ok(path)
    }

    /// Returns the API client
    pub fn api_client(&self) -> Client {
        Client::new(&self.url).token(self.token.as_deref().unwrap_or(""))
    }
}

/// Load from th filesystem or request from the user
pub fn load_or_request() -> Config {
    let cfg = Config::load().unwrap_or_exit();
    match cfg {
        Some(cfg) => cfg,
        None => {
            warn("no config found");
            let url = Text::new("API url:")
                .with_initial_value("http://localhost:3000")
                .prompt()
                .unwrap_or_exit();

            // save to the filesystem
            let cfg = Config { url, token: None };
            let path = cfg.save().unwrap_or_exit();
            success(format!("config saved to {}", path.to_string_lossy()).as_str());

            cfg
        }
    }
}
