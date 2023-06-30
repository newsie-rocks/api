//! This crate contains the REST API service.
//!
//! # Features
//!
//! - **docgen**: activate this flag to generate the OpenAPI specs.
//!
//! # Other tools
//!
//! - **docgen**: The docgen binary generates the OpenAPI documentation.

#![deny(missing_docs)]

use std::sync::Arc;

use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};

pub mod config;
pub mod data;
pub mod http;
pub mod svc;
pub mod trace;

/// Starts the hyper server
pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load the configuration
    let cfg = config::AppConfig::load().await;

    // Create a PostGres connection pool
    // NB: wrapped inside an [Arc] to pass it along
    let postgres_pool = Arc::new(cfg.postgres.pool());

    // Create a Qdrant client
    // NB: wrapped inside an [Arc] to pass it along
    let qdrant_client = Arc::new(cfg.qdrant.client()?);

    // Init the tracing framework
    trace::init_tracer(cfg);

    // Create the HTPTP service
    let service = make_service_fn(|_conn| {
        //  clone the PostGres pool for each connection
        let postgres_pool = postgres_pool.clone();

        //  clone the Qdrant client for each connection
        let qdrant_client = qdrant_client.clone();

        // Define the application context
        let app_ctx = http::AppContext {
            cfg,
            postgres_pool,
            qdrant_client,
        };

        // Service to serve the request
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let ctx = app_ctx.clone();
                http::wrap_app_handler(http::app_handler(ctx, req))
            }))
        }
    });

    // Start the server
    let addr = cfg.server.addr().unwrap();
    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
