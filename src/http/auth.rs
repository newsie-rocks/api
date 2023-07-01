//! Auth handlers

use cookie::Cookie;
use salvo::{oapi::extract::*, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    http::error::HttpError,
    svc::{
        self,
        auth::{NewUser, User},
        Context,
    },
};

/// Authentication cookie name
pub(crate) const AUTH_COOKIE_NAME: &str = "newsie/auth_token";

/// Issues the authentication cookie
fn issue_auth_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(AUTH_COOKIE_NAME, token.to_string())
        .http_only(true)
        // .domain("www.rust-lang.org")
        // .path("/")
        // .secure(true)
        .finish()
}

/// Signup response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SignupRespBody {
    /// JWT auth token
    pub token: String,
    /// User
    pub user: User,
}

/// Handles the signup request
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn signup(
    depot: &mut Depot,
    res: &mut Response,
    body: JsonBody<NewUser>,
) -> Result<Json<SignupRespBody>, HttpError> {
    tracing::trace!("receiving request");

    let new_user = body.into_inner();

    let ctx = depot.obtain::<Context>().unwrap();
    let user = svc::auth::create_user(&ctx, new_user).await?;

    let token = svc::auth::issue_user_token(&ctx, &user)?;
    let auth_cookie = issue_auth_cookie(&token);

    res.status_code(StatusCode::CREATED);
    res.add_cookie(auth_cookie.clone());
    // res.render(Json(SignupRespBody {
    //     token: token.clone(),
    //     user,
    // }));

    Ok(Json(SignupRespBody {
        token: token.clone(),
        user,
    }))
}

/// Login request body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginReqBody {
    /// Email
    email: String,
    /// Password
    password: String,
}

/// Login response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRespBody {
    /// JWT auth token
    token: String,
    /// User
    user: User,
}

/// Handles the login request
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn login(
    depot: &mut Depot,
    res: &mut Response,
    body: JsonBody<LoginReqBody>,
) -> Result<Json<LoginRespBody>, HttpError> {
    let payload = body.into_inner();

    let ctx = depot.obtain::<Context>().unwrap();
    let user = svc::auth::login(&ctx, &payload.email, &payload.password).await?;

    let token = svc::auth::issue_user_token(&ctx, &user)?;
    let auth_cookie = issue_auth_cookie(&token);

    res.status_code(StatusCode::OK);
    res.add_cookie(auth_cookie);

    Ok(Json(LoginRespBody {
        token: token.clone(),
        user,
    }))
}

/// Get user response body
#[derive(Debug, Serialize)]
pub struct GetUserRespBody {
    /// User
    user: User,
}

/// Handles the user query
#[handler]
#[tracing::instrument(skip_all)]
pub async fn get_user(depot: &mut Depot, res: &mut Response) -> Result<(), HttpError> {
    let ctx = depot.obtain::<Context>().unwrap();

    let user = match &ctx.user {
        Some(u) => u.clone(),
        None => {
            return Err(HttpError::Unauthorized(
                "not authenticated".to_string(),
                None,
            ));
        }
    };

    res.render(Json(GetUserRespBody { user }));

    Ok(())
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
