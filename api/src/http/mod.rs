//! REST API

use salvo::{
    oapi::{
        security::{Http, HttpAuthScheme},
        Components, SecurityRequirement, SecurityScheme,
    },
    prelude::*,
};
use tracing::trace;

use crate::{
    config::AppConfig,
    svc::{auth::AuthService, feed::FeedService},
};

pub mod auth;
pub mod feed;
pub mod mdw;

/// Initializes the router
pub fn init_router(cfg: &AppConfig) -> Router {
    // init the services
    let auth_service = AuthService::new(cfg.postgres.new_pool(), cfg.auth.secret.clone());
    let feed_service = FeedService::new(cfg.postgres.new_pool());

    Router::new()
        .hoop(salvo::affix::inject(auth_service))
        .hoop(salvo::affix::inject(feed_service))
        .hoop(mdw::authenticate)
        .get(root)
        .push(Router::with_path("/health").get(healthcheck))
        .push(
            Router::with_path("/auth")
                .push(Router::with_path("/signup").post(auth::signup))
                .push(Router::with_path("/login").post(auth::login))
                .push(
                    Router::with_path("/me")
                        .get(auth::get_me)
                        .patch(auth::update_me)
                        .delete(auth::delete_me),
                ),
        )
        .push(
            Router::with_path("/feeds")
                .get(feed::get_feeds)
                .put(feed::put_feeds),
        )
}

/// Initializes the service
pub fn init_service(cfg: &AppConfig) -> Service {
    let router = init_router(cfg);

    // add the OpenAPI routes
    let openapi = gen_openapi_specs(&router);
    let router = router
        .push(openapi.into_router("/openapi"))
        .push(SwaggerUi::new("/openapi").into_router("/openapi/ui"));

    Service::new(router)
}

/// Generates the OpenAPI specs
pub fn gen_openapi_specs(router: &Router) -> OpenApi {
    let version = env!("CARGO_PKG_VERSION");
    let components = Components::new().add_security_scheme(
        "bearerAuth",
        SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer).bearer_format("JWT")),
    );
    let security_req = SecurityRequirement::new("bearerAuth", vec![] as Vec<String>);
    OpenApi::new("Api", version)
        .components(components)
        .security(vec![security_req])
        .merge_router(router)
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
        let cfg = AppConfig::load();
        let service = init_service(&cfg);
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
            let res = TestClient::get("http://localhost:3000/health")
                .send(&service)
                .await;
            assert_eq!(res.status_code.unwrap(), StatusCode::OK);
        })
        .await;
    }
}
