//! Auth handlers

use std::convert::Infallible;

use hyper::{header, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    http::error::HttpErrorKind,
    svc::{self, Context},
};

use super::{HttpError, HttpRequest, HttpResponse};

/// Signup request body
type SignupReqBody = svc::auth::NewUser;

/// Signup response body
#[derive(Debug, Serialize)]
struct SignupRespBody {
    /// JWT auth token
    token: String,
    /// User
    user: svc::auth::User,
}

/// Handles the signup request
#[tracing::instrument(skip(ctx, req))]
pub async fn handle_signup(ctx: Context, req: HttpRequest) -> Result<HttpResponse, Infallible> {
    tracing::trace!("receiving request");

    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(b) => b,
        Err(err) => {
            let http_error: HttpError = err.into();
            return Ok(http_error.response());
        }
    };
    let new_user = match serde_json::from_slice::<SignupReqBody>(&body) {
        Ok(i) => i,
        Err(err) => {
            let http_error = HttpError::new(HttpErrorKind::InvalidRequest, format!("{err}"), None);
            return Ok(http_error.response());
        }
    };

    let user = match svc::auth::create_user(&ctx, new_user).await {
        Ok(u) => u,
        Err(err) => {
            let http_error: HttpError = err.into();
            return Ok(http_error.response());
        }
    };

    let token = match svc::auth::issue_user_token(&ctx, &user) {
        Ok(t) => t,
        Err(err) => {
            let http_error: HttpError = err.into();
            return Ok(http_error.response());
        }
    };
    let auth_cookie = issue_auth_cookie(&token);

    let body_bytes = serde_json::to_vec(&SignupRespBody { token, user }).unwrap();
    let body = hyper::Body::from(body_bytes);
    let resp = hyper::Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::SET_COOKIE, auth_cookie)
        .body(body)
        .unwrap();

    Ok(resp)
}

/// Login request body
#[derive(Debug, Deserialize)]
struct LoginReqBody {
    /// Email
    email: String,
    /// Password
    password: String,
}

/// Login response body
#[derive(Debug, Serialize)]
struct LoginRespBody {
    /// JWT auth token
    token: String,
    /// User
    user: svc::auth::User,
}

/// Handles the login request
pub async fn handle_login(ctx: Context, req: HttpRequest) -> Result<HttpResponse, Infallible> {
    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(b) => b,
        Err(err) => {
            let http_error: HttpError = err.into();
            return Ok(http_error.response());
        }
    };
    let body = match serde_json::from_slice::<LoginReqBody>(&body) {
        Ok(i) => i,
        Err(e) => {
            let http_error = HttpError::new(HttpErrorKind::InvalidRequest, format!("{e}"), None);
            return Ok(http_error.response());
        }
    };

    let user = match svc::auth::login(&ctx, &body.email, &body.password).await {
        Ok(u) => u,
        Err(err) => {
            let http_error: HttpError = err.into();
            return Ok(http_error.response());
        }
    };

    let token = match svc::auth::issue_user_token(&ctx, &user) {
        Ok(t) => t,
        Err(err) => {
            let http_error: HttpError = err.into();
            return Ok(http_error.response());
        }
    };

    let body_bytes = serde_json::to_vec(&LoginRespBody { token, user }).unwrap();
    let body = hyper::Body::from(body_bytes);
    let res = hyper::Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .status(StatusCode::OK)
        .body(body)
        .unwrap();
    Ok(res)
}

/// Get user response body
#[derive(Debug, Serialize)]
struct GetUserRespBody {
    /// User
    user: svc::auth::User,
}

/// Handles the user query
pub async fn handle_get_user(ctx: Context, _req: HttpRequest) -> Result<HttpResponse, Infallible> {
    let user = match ctx.user {
        Some(u) => u,
        None => {
            let http_error = HttpError::new(
                HttpErrorKind::Unauthorized,
                "not authenticated".to_string(),
                None,
            );
            return Ok(http_error.response());
        }
    };

    let body_bytes = serde_json::to_vec(&GetUserRespBody { user }).unwrap();
    let body = hyper::Body::from(body_bytes);
    let res = hyper::Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .status(StatusCode::OK)
        .body(body)
        .unwrap();
    Ok(res)
}

// /// Handles the user update
// pub async fn handle_update_user(
//     _ctx: Context,
//     _req: HttpRequest,
// ) -> Result<HttpResponse, hyper::Error> {
//     todo!("PATCH /me");
// }

// /// Handles the user deletion
// pub async fn handle_delete_user(
//     _ctx: Context,
//     _req: HttpRequest,
// ) -> Result<HttpResponse, hyper::Error> {
//     todo!("DELETE /me");
// }

/// Authentication cookie name
const AUTH_COOKIE_NAME: &str = "auth_token";

/// Issues the authentication cookie
fn issue_auth_cookie(token: &str) -> String {
    let cookie = cookie::Cookie::build(AUTH_COOKIE_NAME, token)
        .http_only(true)
        // .domain("www.rust-lang.org")
        // .path("/")
        // .secure(true)
        .finish();
    cookie.to_string()
}

/// Parses a http request for an authentication token
pub fn parse_auth_cookie(req: &HttpRequest) -> Result<Option<String>, HttpError> {
    let cookie_h: Option<&hyper::http::HeaderValue> = req.headers().get(hyper::header::COOKIE);
    match cookie_h {
        Some(cookie_v) => {
            let cookie_str = cookie_v.to_str().map_err(|err| {
                HttpError::new(HttpErrorKind::InvalidRequest, format!("{err}"), None)
            })?;
            for c in cookie::Cookie::split_parse(cookie_str) {
                let c = c.unwrap();
                match c.name() {
                    AUTH_COOKIE_NAME => {
                        return Ok(Some(c.value().to_string()));
                    }
                    _ => continue,
                }
            }
            Ok(None)
        }
        None => Ok(None),
    }
}
