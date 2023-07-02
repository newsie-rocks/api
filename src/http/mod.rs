//! Handlers

use salvo::prelude::*;
use tracing::trace;

use crate::{config::AppConfig, svc::Context};

pub mod auth;
pub mod error;
pub mod mdw;

/// Returns the router
pub fn get_router(cfg: &AppConfig) -> Router {
    let ctx = Context::init(cfg);

    let router = Router::new()
        .hoop(salvo::affix::inject(ctx))
        .hoop(mdw::authenticate)
        .get(root)
        .push(Router::with_path("/up").get(healthcheck))
        .push(Router::with_path("/auth/signup").post(auth::signup))
        .push(Router::with_path("/auth/login").post(auth::login))
        .push(
            Router::with_path("/auth/me")
                .get(auth::get_user)
                .patch(auth::update_user)
                .delete(auth::delete_user),
        );

    // set the OpenAPI route
    let version = env!("CARGO_PKG_VERSION");
    let openapi = OpenApi::new("Api", version).merge_router(&router);

    router
        .push(openapi.into_router("/openapi"))
        .push(SwaggerUi::new("/openapi").into_router("/openapi/ui"))
}

/// Returns the service
pub fn get_service(cfg: &AppConfig) -> Service {
    let router = get_router(cfg);
    Service::new(router)
}

/// Serves the root path
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn root() -> &'static str {
    trace!("root");
    "Api service"
}

/// Performs a health check
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn healthcheck() -> &'static str {
    trace!("healthcheck");
    "API is up"
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use super::*;

    use salvo::test::TestClient;

    // Test runner to setup and cleanup a test
    async fn run_test<F>(f: impl Fn(Service) -> F)
    where
        F: Future<Output = ()>,
    {
        let cfg = AppConfig::load().await;
        let service = get_service(cfg);
        f(service).await;
    }

    #[tokio::test]
    async fn test_root() {
        run_test(|service| async move {
            let res = TestClient::get("http://localhost:3000")
                .send(&service)
                .await;
            assert_eq!(res.status_code.unwrap(), StatusCode::OK);
        })
        .await;
    }

    #[tokio::test]
    async fn test_healthcheck() {
        run_test(|service| async move {
            let res = TestClient::get("http://localhost:3000/up")
                .send(&service)
                .await;
            assert_eq!(res.status_code.unwrap(), StatusCode::OK);
        })
        .await;
    }
}
