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
    db::postgres::PostgresClient,
    error::Error,
    svc::{art::ArticleService, auth::AuthService, feed::FeedService},
};

pub mod article;
pub mod auth;
pub mod feed;
pub mod mdw;

/// API services
#[derive(Clone)]
pub struct ApiServices {
    /// Authentication service
    pub auth: AuthService,
    /// Feeds service
    pub feeds: FeedService,
    /// Articles service
    pub art: ArticleService,
}

/// Initializes the HTTP service
pub async fn init_service(cfg: &AppConfig) -> Service {
    let services = init_api_services(cfg).await.unwrap();
    let router = init_router(services).await;

    // add the OpenAPI routes to the service
    let openapi = gen_openapi_specs(&router);
    let router = router
        .push(openapi.into_router("/openapi"))
        .push(SwaggerUi::new("/openapi").into_router("/openapi/ui"));

    Service::new(router)
}

/// Initializes the API services
pub async fn init_api_services(cfg: &AppConfig) -> Result<ApiServices, Error> {
    // init the Postgres client
    let postgres_pool = cfg.postgres.new_pool();
    let postgres_client = PostgresClient::new(postgres_pool);
    postgres_client.init_schema().await?;

    // init the OpenAI client
    let openai_client = cfg.openai.new_client();

    Ok(ApiServices {
        auth: AuthService::new(postgres_client.clone(), cfg.auth.secret.clone()),
        feeds: FeedService::new(postgres_client.clone()),
        art: ArticleService::new(postgres_client, openai_client),
    })
}

/// Initializes the router
pub async fn init_router(services: ApiServices) -> Router {
    Router::new()
        .hoop(salvo::affix::inject(services))
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
        .push(Router::with_path("/articles").put(article::post_articles))
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
    use super::*;

    use salvo::test::TestClient;

    // Test runner to setup and cleanup a test
    async fn setup() -> Service {
        let cfg = AppConfig::load();
        init_service(&cfg).await
    }

    #[tokio::test]
    async fn test_root() {
        let service = setup().await;
        let res = TestClient::get("http://localhost:3000")
            .send(&service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_healthcheck() {
        let service = setup().await;
        let res = TestClient::get("http://localhost:3000/health")
            .send(&service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);
    }
}
