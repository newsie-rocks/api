//! Commands

use anyhow::Error;
use clap::{Parser, Subcommand};
use inquire::{Password, Text};
use newsie_client::NewUser;

use crate::{
    model::Feed,
    svc::Service,
    util::{info, success, ResultExt},
};

/// Runs the program
pub async fn run() -> Result<(), Error> {
    let args = MainArgs::parse();
    match args.commands {
        MainCommands::Config(args) => run_config_cmd(args).await,
        MainCommands::Auth(args) => run_auth_cmd(args).await,
        MainCommands::Feeds(args) => run_feeds_cmd(args).await,
        MainCommands::Read => run_read_cmd().await,
        // MainCommands::Subsc(args) => subsc::run(args).await,
        // MainCommands::Feeds(args) => feed::run(args).await,
    }
}

/// CLI main arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct MainArgs {
    #[command(subcommand)]
    pub commands: MainCommands,
}

/// CLI main commands
#[derive(Subcommand)]
pub enum MainCommands {
    /// Configuration commands
    Config(ConfigArgs),
    /// Authentication and user commands
    Auth(AuthArgs),
    /// Feeds commands
    Feeds(FeedsArgs),
    /// Read the articles
    Read,
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
    /// Shows the CLI configuration
    Show,
    /// Updates the CLI configuration
    Update,
}

/// Runs the config commands
async fn run_config_cmd(args: ConfigArgs) -> Result<(), Error> {
    let service = Service::new()?;
    let mut config = service.get_config()?;
    match args.commands {
        ConfigCommands::Show => {
            println!("Configuration:");
            println!("  - API url: {}", config.api_url);
            println!("  - token: {}", config.token.unwrap_or("none".to_string()));
        }
        ConfigCommands::Update => {
            info("Update the configuration values");
            let api_url = Text::new("API url:")
                .with_initial_value(&config.api_url)
                .prompt()
                .unwrap_or_exit();
            config.api_url = api_url;
            service.update_config(config)?;
            success("configuration updated");
        }
    }
    Ok(())
}

/// Authentication arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct AuthArgs {
    #[command(subcommand)]
    commands: AuthCommands,
}

/// Authentication commands
#[derive(Subcommand)]
pub enum AuthCommands {
    /// Signup
    Signup,
    /// Login
    Login,
    /// Logged in user info
    Me,
    /// Update the logged in user
    Update,
    /// Deletes the logged in user
    Delete,
}

/// Runs the auth commands
async fn run_auth_cmd(args: AuthArgs) -> Result<(), Error> {
    let mut service = Service::new()?;
    match args.commands {
        AuthCommands::Signup => {
            let name = Text::new("Name:").prompt()?;
            let email = Text::new("Email:").prompt()?;
            let password = Password::new("Password:").prompt()?;
            let user = service
                .signup(NewUser {
                    name,
                    email,
                    password,
                })
                .await?;
            success(&format!("Signed up as {}", user.name));
        }
        AuthCommands::Login => {
            info("Enter your login info:");
            let email = Text::new("Email:").prompt()?;
            let password = Password::new("Password:").prompt()?;
            let user = service.login(&email, &password).await?;
            success(&format!("Logged in as {}", user.name));
        }
        AuthCommands::Me => {
            let user = service.me().await?;
            println!("Logged-in user:");
            println!("- name: {}", user.name);
            println!("- email: {}", user.email);
        }
        AuthCommands::Update => {
            todo!()
            // let user = client.me().await.unwrap_or_exit().user;
            // let name = Text::new("Name:")
            //     .with_initial_value(&user.name)
            //     .prompt()
            //     .unwrap_or_exit();
            // let email = Text::new("Email:")
            //     .with_initial_value(&user.email)
            //     .prompt()
            //     .unwrap_or_exit();
            // let password = Password::new("Password:").prompt().unwrap_or_exit();

            // client
            //     .update_me(UserUpdate {
            //         name: Some(name),
            //         email: Some(email),
            //         password: Some(password),
            //     })
            //     .await
            //     .unwrap_or_exit();
            // success("User has been updated");
        }
        AuthCommands::Delete => {
            todo!()
            // client.delete_me().await.unwrap_or_exit();
            // success("User has been deleted");
            // cfg.unset_token(true);
        }
    }
    Ok(())
}

/// Feeds commands
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct FeedsArgs {
    #[command(subcommand)]
    commands: FeedsCommands,
}

/// Feed commands
#[derive(Subcommand)]
pub enum FeedsCommands {
    /// List all the feeds
    Ls,
    /// Adds a feed
    Add {
        /// Feed url
        url: String,
        /// Feed name
        #[arg(long, short)]
        name: Option<String>,
        /// Folder name
        #[arg(long, short)]
        folder: Option<String>,
    },
    /// Removes feeds
    Rm {
        /// Feeds urls
        urls: Vec<String>,
    },
}

/// Runs the feeds commands
async fn run_feeds_cmd(args: FeedsArgs) -> Result<(), Error> {
    let mut service = Service::new()?;
    match args.commands {
        FeedsCommands::Ls => {
            let feeds = service.get_feeds().await?;
            println!("FEEDS:");
            for feed in &feeds {
                println!("  - {}", feed.url);
            }
        }
        FeedsCommands::Add { url, name, folder } => {
            let feed = Feed { url, name, folder };
            service.add_feeds(vec![feed]).await?;
            success("feed added");
        }
        FeedsCommands::Rm { urls } => {
            service.remove_feeds(urls).await?;
            success("feed(s) removed");
        }
    }
    Ok(())
}

/// Runs the read command
async fn run_read_cmd() -> Result<(), Error> {
    let service = Service::new()?;
    let feeds = service.get_feeds().await?;
    for feed in feeds {
        println!("FEED: {}", feed.url);
        let articles = service.get_articles(&feed).await?;
        for article in articles {
            println!("  - {}", article.url);
        }
    }
    println!("OK");
    Ok(())
}
