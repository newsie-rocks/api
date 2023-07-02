//! Middlewares

use crate::{
    config::AppConfig,
    svc::{self, Context},
};

use salvo::{hyper::header::AUTHORIZATION, prelude::*};

use super::{auth::AUTH_COOKIE_NAME, error::HttpError};

/// Middleware to authenticate the user
#[handler]
pub async fn authenticate(req: &mut Request, depot: &mut Depot) -> Result<(), HttpError> {
    // NB: Context must be set before the user is extracted
    let ctx = depot.obtain::<Context>().unwrap();

    // Extract the auth token from the AUTHORIZATION header
    let mut token = None;
    if let Some(v) = req.headers().get(AUTHORIZATION) {
        match v.to_str() {
            Ok(s) => match s.strip_prefix("Bearer ") {
                Some(s) => token = Some(s.to_string()),
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
        }
    }

    // If undefined, try extracting the token from the cookie
    if token.is_none() {
        if let Some(cookie) = req.cookie(AUTH_COOKIE_NAME) {
            token = Some(cookie.value().to_string());
        };
    }

    // Read the user and populate the context
    if let Some(t) = token {
        let user = svc::auth::read_user_with_token(ctx, t.as_str()).await?;
        let cfg = AppConfig::load().await;
        let mut new_ctx = Context::init(cfg);
        new_ctx.user = user;
        depot.inject(new_ctx);
    }

    Ok(())
}
