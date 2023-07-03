//! Generates the OpenAPI documentation

use newsie_api::config::AppConfig;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    let cfg = AppConfig::load().await;
    let router = newsie_api::http::get_router(&cfg);
    let version = env!("CARGO_PKG_VERSION");
    let openapi = OpenApi::new("Api", version).merge_router(&router);
    println!("{}", openapi.to_yaml().unwrap());
}
