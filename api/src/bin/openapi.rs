//! Generates the OpenAPI documentation

use newsie_api::{
    config::AppConfig,
    http::{gen_openapi_specs, init_router},
};

fn main() {
    let cfg = AppConfig::load();
    let router = init_router(&cfg);
    let openapi = gen_openapi_specs(&router);
    println!("{}", openapi.to_yaml().unwrap());
}
