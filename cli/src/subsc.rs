//! Subscription

use clap::{Parser, Subcommand};
use inquire::Select;
use newsie_client::{Subscription, SubscriptionUpdate};

use crate::{
    config::Config,
    util::{info, success, OptionExt, ResultExt},
};

/// Subscription commands
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct SubscArgs {
    #[command(subcommand)]
    commands: SubscCommands,
}

/// Subscription commands
#[derive(Subcommand)]
pub enum SubscCommands {
    /// Show the current subscription
    Show,
    /// Update the subscription
    Update,
}

/// Runs the subscription commands
pub async fn run(args: SubscArgs) {
    let cfg = Config::load()
        .unwrap_or_exit()
        .unwrap_or_exit("config not set");
    let client = cfg.api_client();
    let user = client.me().await.unwrap_or_exit().user;

    match args.commands {
        SubscCommands::Show => {
            println!("User subscription: {}", user.subscription);
        }
        SubscCommands::Update => {
            info(&format!("current subscription: {}", user.subscription));
            let options: Vec<&str> = vec!["Free Tier", "Mid Tier"];
            let selected_subsc = Select::new("Select your new subscription:", options.clone())
                .prompt()
                .unwrap_or_exit();

            let subscription_update = match selected_subsc {
                "Free Tier" => SubscriptionUpdate {
                    subscription: Subscription::Free,
                },
                "Mid Tier" => SubscriptionUpdate {
                    subscription: Subscription::Mid,
                },
                _ => unreachable!(),
            };
            let _user = client
                .update_subscription(subscription_update)
                .await
                .unwrap_or_exit();
            success("updated subscription");
        }
    }
}
