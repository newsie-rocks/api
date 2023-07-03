//! Middlewares

use salvo::{hyper::header::AUTHORIZATION, prelude::*};
use tracing::trace;

use crate::svc::auth::AuthService;

use super::{auth::AUTH_COOKIE_NAME, error::HttpError};

/// Middleware to authenticate the user
#[handler]
pub async fn authenticate(req: &mut Request, depot: &mut Depot) -> Result<(), HttpError> {
    // NB: Context must be set before the user is extracted
    let auth = depot.obtain::<AuthService>().unwrap();

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
    if let Some(token) = token {
        trace!(token, "auth token");
        let user = auth.read_with_token(&token).await?;
        trace!(?user, "auth user");
        if let Some(user) = user {
            depot.inject(user);
        }
    } else {
        trace!(token, "not authenticated");
    }

    Ok(())
}
