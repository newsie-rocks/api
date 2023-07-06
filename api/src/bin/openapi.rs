//! Generates the OpenAPI documentation

use newsie_api::config::AppConfig;
use salvo::prelude::*;

fn main() {
    let cfg = AppConfig::load();
    let router = newsie_api::http::get_router(&cfg);
    let version = env!("CARGO_PKG_VERSION");
    let openapi = OpenApi::new("Api", version).merge_router(&router);
    let openapi_str = openapi.to_yaml().unwrap();
    println!("{openapi_str}");
}
