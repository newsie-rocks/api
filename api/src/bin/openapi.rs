//! Generates the OpenAPI documentation

use newsie_api::{
    config::AppConfig,
    http::{gen_openapi_specs, init_api_services, init_router},
};

#[tokio::main]
async fn main() {
    let cfg = AppConfig::load();
    let api_services = init_api_services(&cfg).await.unwrap();
    let router = init_router(api_services).await;
    let openapi = gen_openapi_specs(&router);
    println!("{}", openapi.to_yaml().unwrap());
}
