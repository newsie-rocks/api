//! CLI client

use clap::{Parser, Subcommand};

pub mod auth;
pub mod config;
pub mod util;

/// Newsie CLI client
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}

/// CLI commands
#[derive(Subcommand)]
enum Commands {
    /// Configuration commands
    Config(config::ConfigArgs),
    /// Authentication and user commands
    Auth(auth::AuthArgs),
}

#[tokio::main]
async fn main() {
    println!();

    let args = Args::parse();
    match args.commands {
        Commands::Config(args) => config::run(args).await,
        Commands::Auth(args) => auth::run(args).await,
    }
}
