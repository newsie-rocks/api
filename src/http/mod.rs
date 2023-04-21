//! HTTP handlers

use hyper::{header::CONTENT_TYPE, Body, Method};
use serde::Serialize;

use crate::svc::Context;

pub mod auth;
pub mod mdw;

/// HTTP request
pub type Request = hyper::Request<Body>;

/// HTTP response
pub type Response = hyper::Response<Body>;

/// Application context
#[derive(Debug, Clone)]
pub struct AppContext {
    /// Authentication secret
    pub auth_secret: String,
    /// DB pool
    pub db_pool: deadpool_postgres::Pool,
}

/// Application handlers
pub async fn app_handler(app_ctx: AppContext, req: Request) -> Result<Response, hyper::Error> {
    // Define the request context
    let mut ctx = Context {
        auth_secret: app_ctx.auth_secret.clone(),
        db_pool: app_ctx.db_pool.clone(),
        user: None,
    };

    // Pass middleware(s)
    match self::mdw::extract_user(&mut ctx, &req).await {
        Ok(user) => user,
        Err(err) => match err {
            mdw::MdwError::BadRequest { message } => {
                let api_error = ApiError::InvalidInput { message };
                return Ok(api_error.response());
            }
            mdw::MdwError::Internal { message } => {
                let api_error = ApiError::Server {
                    message: "Internal server error".to_string(),
                };
                return Ok(api_error.response());
            }
            mdw::MdwError::InvalidToken { message: _ } => {}
        },
    };

    // Process requests
    match (req.method(), req.uri().path()) {
        // -- BASE --
        (&Method::GET, "/") => {
            let body = Body::from("API is up");
            Ok(hyper::Response::builder()
                .header(CONTENT_TYPE, "text/plain")
                .body(body)
                .unwrap())
        }
        // -- AUTH --
        (&Method::POST, "/auth/signup") => self::auth::handle_signup(ctx, req).await,
        (&Method::POST, "/auth/login") => self::auth::handle_login(ctx, req).await,
        (&Method::GET, "/auth/me") => self::auth::handle_get_user(ctx, req).await,
        (&Method::PATCH, "/auth/me") => self::auth::handle_update_user(ctx, req).await,
        (&Method::DELETE, "/auth/me") => self::auth::handle_delete_user(ctx, req).await,
        // -- FEEDS --
        (&Method::GET, "/feeds") => {
            todo!("Get all feeds")
        }
        (&Method::POST, "/feeds") => {
            todo!("Add a feed")
        }
        (&Method::DELETE, "/feeds") => {
            todo!("Delete a feed")
        }
        // 404
        _ => {
            let body = Body::from("mehhhh, nothing here");
            Ok(hyper::Response::builder()
                .status(404)
                .header(CONTENT_TYPE, "text/plain")
                .body(body)
                .unwrap())
        }
    }
}

/// API error
#[derive(Debug, thiserror::Error, Serialize)]
pub enum ApiError {
    /// Invalid input
    #[error("invalid input:  {message:?}")]
    InvalidInput {
        /// Error message
        message: String,
    },
    /// Server error
    #[error("server error:  {message:?}")]
    Server {
        /// Error message
        message: String,
    },
}

impl ApiError {
    /// Converts to an HTTP response
    pub fn response(self) -> Response {
        let status = match &self {
            ApiError::InvalidInput { message: _ } => 400,
            ApiError::Server { message: _ } => 500,
        };

        let body_json = serde_json::to_string(&self).unwrap();
        let body = Body::from(body_json);

        hyper::Response::builder()
            .status(status)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap()
    }
}
