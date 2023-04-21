use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};

use api::{
    config::AppConfig,
    http::{self, AppContext},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load the configuration
    let cfg = AppConfig::load().await;

    // Create the service
    let service = make_service_fn(|_conn| async {
        // Create a db connection pool
        let db_pool = cfg.db.pool();

        // Define the application context
        let app_ctx = AppContext {
            auth_secret: cfg.auth.secret.clone(),
            db_pool,
        };

        // Service to serve the request
        Ok::<_, hyper::Error>(service_fn(move |req| {
            http::app_handler(app_ctx.clone(), req)
        }))
    });

    // Start the server
    let addr = cfg.server.addr().unwrap();
    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
