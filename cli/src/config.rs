//! Configuration

use std::{fs, path::PathBuf, process::exit};

use anyhow::Result;
use clap::{Parser, Subcommand};
use inquire::{Confirm, Text};
use newsie_client::Client;
use serde::{Deserialize, Serialize};

use crate::util::{success, warn, OptionExt, ResultExt};

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

    /// Returns the path
    fn path(&self) -> PathBuf {
        Self::default_path()
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

    /// Initializes the configuration
    pub fn init() -> Self {
        if Config::load().unwrap_or_exit().is_some() {
            match Confirm::new("Config is already set, do you want to overwrite it?")
                .with_default(false)
                .prompt()
                .unwrap_or_exit()
            {
                true => {}
                false => {
                    warn("config has not been modified");
                    exit(1);
                }
            }
        };

        let url = Text::new("API url:")
            .with_initial_value("http://localhost:3000")
            .prompt()
            .unwrap_or_exit();

        // save to the filesystem
        let cfg = Config { url, token: None };
        cfg.save().unwrap_or_exit();
        success(format!("config saved to {}", cfg.path().to_string_lossy()).as_str());
        cfg
    }

    /// Saves the config to the filesystem
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
        // NB : we need to create the parent directory
        fs::create_dir_all(path.parent().unwrap())?;
        let cfg_str = toml::to_string(self)?;
        fs::write(&path, cfg_str)?;
        Ok(())
    }

    /// Removes the config from the filesystem
    pub fn destroy(&self) -> Result<()> {
        let path = Self::default_path();
        Ok(fs::remove_file(path)?)
    }

    /// Sets the token
    pub fn set_token(&mut self, token: &str, save: bool) {
        self.token = Some(token.to_owned());
        if save {
            self.save().unwrap_or_exit();
        }
    }

    /// Removes the token
    pub fn unset_token(&mut self, save: bool) {
        self.token = None;
        if save {
            self.save().unwrap_or_exit();
        }
    }

    /// Returns the API client
    pub fn api_client(&self) -> Client {
        Client::new(&self.url).token(self.token.clone())
    }
}

/// Configuration commands
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct ConfigArgs {
    #[command(subcommand)]
    commands: ConfigCommands,
}

/// Configuration commands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initializes the configuration
    Init,
    /// List the configuration
    Ls,
    /// Resets the configuration
    Reset,
}

/// Runs the auth commands
pub async fn run(args: ConfigArgs) {
    match args.commands {
        ConfigCommands::Init => {
            Config::init();
        }
        ConfigCommands::Ls => {
            let cfg = Config::load()
                .unwrap_or_exit()
                .unwrap_or_exit("config not set");
            println!("CLI client info");
            println!("  - endpoint: {}", cfg.url);
            println!("  - token: {}", cfg.token.unwrap_or("none".to_string()));
        }
        ConfigCommands::Reset => {
            let cfg = Config::load()
                .unwrap_or_exit()
                .unwrap_or_exit("config not set");
            cfg.destroy().unwrap_or_exit();

            Config::init();
        }
    }
}
