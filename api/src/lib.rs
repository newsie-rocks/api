//! This crate contains the REST API service.
//!
//! # Features
//!
//! - **None**: document features here
//!
//! # Other binaries
//!
//! - **docgen**: The docgen binary generates the OpenAPI documentation.

#![deny(missing_docs)]

use crate::config::AppConfig;
use salvo::prelude::*;

pub mod config;
pub mod db;
pub mod error;
pub mod http;
pub mod mdl;
pub mod svc;
pub mod trace;

/// Starts the server
pub async fn start_server(cfg: AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // init the tracing framework
    trace::init_tracer(&cfg);

    // create the HTTP service
    let service = http::init_service(&cfg).await;

    // start the server
    let addr = cfg.server.addr().unwrap();
    let acceptor = TcpListener::new(addr).bind().await;
    eprintln!();
    eprintln!("Listening on http://{}", addr);
    Server::new(acceptor).serve(service).await;
    Ok(())
}
