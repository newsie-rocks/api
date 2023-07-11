//! Commands

use anyhow::Error;
use clap::{Parser, Subcommand};
use inquire::{Password, Text};
use newsie_client::NewUser;

use crate::{
    svc::Service,
    util::{info, success, ResultExt},
};

/// Runs the program
pub async fn run() -> Result<(), Error> {
    let args = MainArgs::parse();
    match args.commands {
        MainCommands::Config(args) => run_config_cmd(args).await,
        MainCommands::Auth(args) => run_auth_cmd(args).await,
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
    // /// Subscription commands
    // Subsc(subsc::SubscArgs),
    // /// Feeds commands
    // Feeds(feed::FeedsArgs),
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
    /// Shows the configuration
    Show,
    /// Updates the configuration
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
pub async fn run_auth_cmd(args: AuthArgs) -> Result<(), Error> {
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
            todo!()
            // let email = Text::new("Email:").prompt().unwrap_or_exit();
            // let password = Password::new("Password:").prompt().unwrap_or_exit();
            // let res = client.login(&email, &password).await.unwrap_or_exit();

            // cfg.set_token(&res.token, true);
            // success(&format!("Logged-in as {}", res.user.name));
        }
        AuthCommands::Me => {
            todo!()
            // let res = client.me().await.unwrap_or_exit();
            // let user = res.user;

            // println!("Logged-in user:");
            // println!("- name: {}", user.name);
            // println!("- email: {}", user.email);
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
