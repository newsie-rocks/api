//! HTTP handlers

use std::{convert::Infallible, future::Future, panic::AssertUnwindSafe, sync::Arc};

use futures::future::FutureExt;
use hyper::{header::CONTENT_TYPE, Body, Method, StatusCode};

use crate::svc::Context;

use self::error::HttpError;

pub mod auth;
pub mod error;
pub mod mdw;

/// HTTP request
pub type HttpRequest = hyper::Request<Body>;

/// HTTP response
pub type HttpResponse = hyper::Response<Body>;

/// Application context
#[derive(Debug, Clone)]
pub struct AppContext {
    /// Authentication secret
    pub auth_secret: String,
    /// DB pool
    pub db_pool: Arc<deadpool_postgres::Pool>,
}

/// Application handlers
pub async fn app_handler(
    app_ctx: AppContext,
    req: HttpRequest,
) -> Result<HttpResponse, Infallible> {
    // Define the request context
    let mut ctx = Context {
        auth_secret: app_ctx.auth_secret.clone(),
        db_pool: app_ctx.db_pool.clone(),
        user: None,
    };

    // Pass middleware(s)
    match self::mdw::extract_user(&mut ctx, &req).await {
        Ok(user) => user,
        Err(err) => {
            let http_error: HttpError = err.into();
            return Ok(http_error.response());
        }
    };

    // Process requests
    match (req.method(), req.uri().path()) {
        // -- AUTH --
        (&Method::POST, "/auth/signup") => self::auth::handle_signup(ctx, req).await,
        (&Method::POST, "/auth/login") => self::auth::handle_login(ctx, req).await,
        (&Method::GET, "/auth/me") => self::auth::handle_get_user(ctx, req).await,
        // (&Method::PATCH, "/auth/me") => self::auth::handle_update_user(ctx, req).await,
        // (&Method::DELETE, "/auth/me") => self::auth::handle_delete_user(ctx, req).await,
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
        // -- OTHER --
        (&Method::GET, "/") => handle_hello(ctx, req).await,
        _ => handle_404(ctx, req).await,
    }
}

/// Wraps the app handler
///
/// This is used to catch a panic inside the handler
pub async fn wrap_app_handler(
    fut: impl Future<Output = Result<HttpResponse, Infallible>>,
) -> Result<HttpResponse, Infallible> {
    // 1. Wrap the future in AssertUnwindSafe, to make the compiler happy
    //    and allow us doing this. The wrapper also implements `Future`
    //    and delegates `poll` inside.
    // 2. Turn panics falling out of the `poll` into errors. Note that we
    //    get `Result<Result<_, _>, _>` thing here.
    match AssertUnwindSafe(fut).catch_unwind().await {
        // Here we unwrap just the outer panic-induced `Result`, returning
        // the inner `Result`
        Ok(response) => response,
        Err(_panic) => Ok(hyper::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("We screwed up, sorry!"))
            .unwrap()),
    }
}

/// Handles the base route
#[tracing::instrument]
#[cfg_attr(feature = "docgen", utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "API is up"),
        (status = 500, description = "API is unavailable")
    )
))]
pub async fn handle_hello(_ctx: Context, _req: HttpRequest) -> Result<HttpResponse, Infallible> {
    tracing::trace!("receiving request");

    let body = Body::from("API is up");
    Ok(hyper::Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain")
        .body(body)
        .unwrap())
}

/// Handles the base route
#[tracing::instrument(skip_all)]
async fn handle_404(_ctx: Context, _req: HttpRequest) -> Result<HttpResponse, Infallible> {
    tracing::trace!("receiving request with invalid URL path");
    let body = Body::from("mehhhh, nothing here");
    Ok(hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .body(body)
        .unwrap())
}
