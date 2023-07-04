//! Authentication commands

use clap::{Parser, Subcommand};
use inquire::{Password, Text};
use newsie_client::{NewUser, UserFields};

use crate::{
    config::Config,
    util::{success, OptionExt, ResultExt},
};

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
    Info,
    /// Update the logged in user
    Update,
    /// Deletes the logged in user
    Delete,
}

/// Runs the auth commands
pub async fn run(args: AuthArgs) {
    let mut cfg = Config::load()
        .unwrap_or_exit()
        .unwrap_or_exit("config not set");
    let mut client = cfg.api_client();

    match args.commands {
        AuthCommands::Signup => {
            let name = Text::new("Name:").prompt().unwrap_or_exit();
            let email = Text::new("Email:").prompt().unwrap_or_exit();
            let password = Password::new("Password:").prompt().unwrap_or_exit();
            let res = client
                .signup(NewUser {
                    name,
                    email,
                    password,
                })
                .await
                .unwrap_or_exit();

            cfg.set_token(&res.token, true);
            success(&format!("Signed up as {}", res.user.name));
        }
        AuthCommands::Login => {
            let email = Text::new("Email:").prompt().unwrap_or_exit();
            let password = Password::new("Password:").prompt().unwrap_or_exit();
            let res = client.login(&email, &password).await.unwrap_or_exit();

            cfg.set_token(&res.token, true);
            success(&format!("Logged-in as {}", res.user.name));
        }
        AuthCommands::Info => {
            let res = client.me().await.unwrap_or_exit();
            let user = res.user;

            println!("Logged-in user:");
            println!("- name: {}", user.name);
            println!("- email: {}", user.email);
        }
        AuthCommands::Update => {
            let user = client.me().await.unwrap_or_exit().user;

            let name = Text::new("Name:")
                .with_initial_value(&user.name)
                .prompt()
                .unwrap_or_exit();
            let email = Text::new("Email:")
                .with_initial_value(&user.name)
                .prompt()
                .unwrap_or_exit();
            let password = Password::new("Password:").prompt().unwrap_or_exit();

            client
                .update_me(UserFields {
                    name: Some(name),
                    email: Some(email),
                    password: Some(password),
                })
                .await
                .unwrap_or_exit();
        }
        AuthCommands::Delete => todo!(),
    }
}
