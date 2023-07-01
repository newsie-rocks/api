//! Handlers

use salvo::prelude::*;

pub mod auth;
pub mod error;
pub mod mdw;

/// Returns the router
pub fn get_router() -> Router {
    let router = Router::new()
        .hoop(mdw::add_context)
        .get(root)
        .push(Router::with_path("/up").get(healthcheck))
        .push(Router::with_path("/auth/signup").post(auth::signup))
        .push(Router::with_path("/auth/login").post(auth::login))
        .push(Router::with_path("/auth/me").get(auth::get_user));

    // set the OpenAPI route
    let version = env!("CARGO_PKG_VERSION");
    let openapi = OpenApi::new("Api", version).merge_router(&router);
    let router = router
        .push(openapi.into_router("/openapi"))
        .push(SwaggerUi::new("/openapi").into_router("/openapi/ui"));

    router
}

/// Serves the root path
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn root() -> &'static str {
    tracing::trace!("root");
    "Api service"
}

/// Performs a health check
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn healthcheck() -> &'static str {
    tracing::trace!("healthcheck");
    "API is up"
}
