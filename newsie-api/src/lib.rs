//! This crate contains the REST API service.
//!
//! # Features
//!
//! - **TBD**: document features here
//!
//! # Other binaries
//!
//! - **docgen**: The docgen binary generates the OpenAPI documentation.

#![deny(missing_docs)]

use salvo::prelude::*;

pub mod config;
pub mod data;
pub mod http;
pub mod svc;
pub mod trace;

/// Starts the hyper server
pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load the configuration
    let cfg = config::AppConfig::load().await;

    // Init the tracing framework
    trace::init_tracer(&cfg);

    // Create the HTTP service
    let service = http::get_service(&cfg);

    // Start the server
    let addr = cfg.server.addr().unwrap();
    let acceptor = TcpListener::new(addr).bind().await;
    println!("Listening on http://{}", addr);
    Server::new(acceptor).serve(service).await;
    Ok(())
}