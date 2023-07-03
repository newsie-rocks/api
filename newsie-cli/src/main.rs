//! CLI client

use clap::{Parser, Subcommand};

pub mod auth;
pub mod config;
pub mod info;
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
    /// Client info
    Info,
    /// Authentication and user commands
    Auth(auth::AuthArgs),
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.commands {
        Commands::Info => info::get_info(),
        Commands::Auth(args) => auth::run(args).await,
    }
}
