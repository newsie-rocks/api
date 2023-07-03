//! Authentication commands

use clap::{Parser, Subcommand};
use inquire::{Password, Text};

use crate::{config, util::ResultExt};

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
    match args.commands {
        AuthCommands::Signup => {
            let cfg = config::load_or_request();
            let email = Text::new("Email:").prompt().unwrap_or_exit();
            let password = Password::new("Password:").prompt().unwrap_or_exit();
        }
        AuthCommands::Login => todo!(),
        AuthCommands::Info => todo!(),
        AuthCommands::Update => todo!(),
        AuthCommands::Delete => todo!(),
    }
}
