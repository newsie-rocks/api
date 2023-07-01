//! Generates the OpenAPI documentation

use salvo::prelude::*;

fn main() {
    let router = api::http::get_router();
    let version = env!("CARGO_PKG_VERSION");
    let openapi = OpenApi::new("Api", version).merge_router(&router);
    println!("{}", openapi.to_yaml().unwrap());
}
