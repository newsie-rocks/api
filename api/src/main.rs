//! Server

use newsie_api::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = AppConfig::load();
    newsie_api::start_server(cfg).await
}
