//! Feeds

use clap::{Parser, Subcommand};

use crate::{
    config::Config,
    util::{OptionExt, ResultExt},
};

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
    /// Sync the feeds
    Sync,
}

/// Runs the feeds commands
pub async fn run(args: FeedsArgs) {
    let cfg = Config::load()
        .unwrap_or_exit()
        .unwrap_or_exit("config not set");
    let client = cfg.api_client();

    match args.commands {
        FeedsCommands::Ls => {
            // println!("User subscription: {}", user.subscription);
        }
        FeedsCommands::Sync => {
            // info(&format!("current subscription: {}", user.subscription));
            // let options: Vec<&str> = vec!["Free Tier", "Mid Tier"];
            // let selected_subsc = Select::new("Select your new subscription:", options.clone())
            //     .prompt()
            //     .unwrap_or_exit();

            // let subscription_update = match selected_subsc {
            //     "Free Tier" => SubscriptionUpdate {
            //         subscription: Subscription::Free,
            //     },
            //     "Mid Tier" => SubscriptionUpdate {
            //         subscription: Subscription::Mid,
            //     },
            //     _ => unreachable!(),
            // };
            // let _user = client
            //     .update_subscription(subscription_update)
            //     .await
            //     .unwrap_or_exit();
            // success("updated subscription");
            todo!()
        }
    }
}
