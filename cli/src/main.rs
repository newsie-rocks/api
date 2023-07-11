//! CLI client

use crate::util::ResultExt;

pub mod cmd;
pub mod svc;
pub mod util;

// pub mod auth;
// pub mod feed;
// pub mod subsc;

#[tokio::main]
async fn main() {
    println!();

    cmd::run().await.unwrap_or_exit();
}
