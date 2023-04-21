//! Auth handlers

use serde::Deserialize;

use crate::svc::Context;

use super::{ApiError, Request, Response};

/// Signup input
#[derive(Debug, Deserialize)]
struct SignupInput {
    /// Email
    email: String,
    /// Password
    password: String,
}

/// Handles the signup request
pub async fn handle_signup(_ctx: Context, req: Request) -> Result<Response, hyper::Error> {
    let body = hyper::body::to_bytes(req.into_body()).await?;
    let input = match serde_json::from_slice::<SignupInput>(&body) {
        Ok(i) => i,
        Err(e) => {
            let api_error = ApiError::InvalidInput {
                message: format!("{e}"),
            };
            return Ok(api_error.response());
        }
    };

    todo!("signup");

    // Ok(Response::new(Body::from("Got JSON object!")))
}

/// Login input
#[derive(Debug, Deserialize)]
struct LoginInput {
    /// Email
    email: String,
    /// Password
    password: String,
}

/// Handles the login request
pub async fn handle_login(ctx: Context, req: Request) -> Result<Response, hyper::Error> {
    let body = hyper::body::to_bytes(req.into_body()).await?;
    let input = match serde_json::from_slice::<LoginInput>(&body) {
        Ok(i) => i,
        Err(e) => {
            let api_error = ApiError::InvalidInput {
                message: format!("{e}"),
            };
            return Ok(api_error.response());
        }
    };

    todo!("login");
}

/// Handles the user query
pub async fn handle_get_user(ctx: Context, req: Request) -> Result<Response, hyper::Error> {
    todo!("GET /me");
}

/// Handles the user update
pub async fn handle_update_user(ctx: Context, req: Request) -> Result<Response, hyper::Error> {
    todo!("PATCH /me");
}

/// Handles the user deletion
pub async fn handle_delete_user(ctx: Context, req: Request) -> Result<Response, hyper::Error> {
    todo!("DELETE /me");
}
