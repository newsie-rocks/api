//! CLI client

use crate::util::ResultExt;

mod cmd;
mod db;
mod model;
mod svc;
mod util;

// pub mod auth;
// pub mod feed;
// pub mod subsc;

#[tokio::main]
async fn main() {
    println!();

    cmd::run().await.unwrap_or_exit();
}
