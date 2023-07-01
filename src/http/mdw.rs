//! Middlewares

use std::sync::Arc;

use crate::{
    config::AppConfig,
    svc::{self, Context},
};

use qdrant_client::prelude::QdrantClient;
use salvo::{hyper::header::AUTHORIZATION, prelude::*};
use tokio::sync::OnceCell;

use super::{auth::AUTH_COOKIE_NAME, error::HttpError};

/// PostGres pool global instance
static POSTGRES_POOL: OnceCell<deadpool_postgres::Pool> = OnceCell::const_new();

/// Qdrant client global instance
static QDRANT_CLIENT: OnceCell<Arc<QdrantClient>> = OnceCell::const_new();

/// Middleware to add the context
#[handler]
pub async fn add_context(req: &mut Request, depot: &mut Depot) -> Result<(), HttpError> {
    let cfg = AppConfig::load().await;

    // add the PostGres pool
    let postgres_pool = POSTGRES_POOL
        .get_or_init(|| async { cfg.postgres.pool() })
        .await;

    // add the Qddrant client
    let qdrant_client = QDRANT_CLIENT
        .get_or_init(|| async {
            let client = cfg.qdrant.client().unwrap();
            Arc::new(client)
        })
        .await;

    // set the context
    let mut ctx = Context {
        auth_secret: cfg.auth.secret.clone(),
        postgres_pool: postgres_pool.clone(),
        qdrant_client: qdrant_client.clone(),
        user: None,
    };

    // add the user
    extract_user(&mut ctx, req).await?;

    // inject the context
    depot.inject(ctx);

    Ok(())
}

/// Extract the user from the request
async fn extract_user(ctx: &mut Context, req: &Request) -> Result<(), HttpError> {
    // Extract the auth token from the AUTHORIZATION header
    let mut token = match req.headers().get(AUTHORIZATION) {
        Some(v) => match v.to_str() {
            Ok(s) => match s.strip_prefix("Bearer") {
                Some(s) => Some(s.to_string()),
                None => {
                    return Err(HttpError::BadRequest(
                        "Invalid authorization header".to_string(),
                        None,
                    ))
                }
            },
            Err(err) => {
                return Err(HttpError::BadRequest(
                    "Invalid authorization header".to_string(),
                    Some(err.to_string()),
                ))
            }
        },
        None => None,
    };

    // If undefined, try extracting the token from the cookie
    if token.is_none() {
        if let Some(cookie) = req.cookie(AUTH_COOKIE_NAME) {
            token = Some(cookie.value().to_string());
        };
    }

    // Read the user and populate the context
    if let Some(t) = token {
        let user = svc::auth::read_user_with_token(ctx, t.as_str()).await?;
        ctx.user = user;
    }

    Ok(())
}
