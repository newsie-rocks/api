//! Auth handlers

use cookie::Cookie;
use salvo::{oapi::extract::*, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    http::error::HttpError,
    svc::{
        self,
        auth::{NewUser, User, UserFields},
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
    trace!("received request");

    let new_user = body.into_inner();

    let ctx = depot.obtain::<Context>().unwrap();
    let user = svc::auth::create_user(ctx, new_user).await?;

    let token = svc::auth::issue_user_token(ctx, &user)?;
    let auth_cookie = issue_auth_cookie(&token);

    res.status_code(StatusCode::CREATED);
    res.add_cookie(auth_cookie.clone());

    Ok(Json(SignupRespBody {
        token: token.clone(),
        user,
    }))
}

/// Login request body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginReqBody {
    /// Email
    pub email: String,
    /// Password
    pub password: String,
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
    trace!("received request");

    let payload = body.into_inner();

    let ctx = depot.obtain::<Context>().unwrap();
    let user = svc::auth::login(ctx, &payload.email, &payload.password).await?;

    let token = svc::auth::issue_user_token(ctx, &user)?;
    let auth_cookie = issue_auth_cookie(&token);

    res.status_code(StatusCode::OK);
    res.add_cookie(auth_cookie);

    Ok(Json(LoginRespBody {
        token: token.clone(),
        user,
    }))
}

/// Get user response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetUserRespBody {
    /// User
    user: User,
}

/// Fetches the current user
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn get_user(depot: &mut Depot) -> Result<Json<GetUserRespBody>, HttpError> {
    trace!("received request");

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

    Ok(Json(GetUserRespBody { user }))
}

/// Updates the current user
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn update_user(
    depot: &mut Depot,
    body: JsonBody<UserFields>,
) -> Result<Json<GetUserRespBody>, HttpError> {
    trace!("received request");

    let ctx = depot.obtain::<Context>().unwrap();

    let user_id = match &ctx.user {
        Some(u) => u.id,
        None => {
            return Err(HttpError::Unauthorized(
                "not authenticated".to_string(),
                None,
            ));
        }
    };
    let user = crate::svc::auth::update_user(ctx, user_id, body.into_inner()).await?;

    Ok(Json(GetUserRespBody { user }))
}

/// Deletes a user
///
/// The ID is retrieved from the token
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn delete_user(depot: &mut Depot) -> Result<(), HttpError> {
    trace!("received request");

    let ctx = depot.obtain::<Context>().unwrap();

    let user_id = match &ctx.user {
        Some(u) => u.id,
        None => {
            return Err(HttpError::Unauthorized(
                "not authenticated".to_string(),
                None,
            ));
        }
    };
    crate::svc::auth::delete_user(ctx, user_id).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use super::*;

    use fake::{
        faker::{internet::en::FreeEmail, name::en::Name},
        Fake,
    };
    use salvo::{
        hyper::header::AUTHORIZATION,
        test::{ResponseExt, TestClient},
        Service,
    };

    use crate::{
        config::AppConfig,
        http::get_service,
        svc::auth::{issue_user_token, NewUser},
    };

    // Test runner to setup and cleanup a test
    async fn run_test<F>(f: impl Fn(Service, Context, String) -> F)
    where
        F: Future<Output = (Service, Context, String)>,
    {
        // setup
        let cfg = AppConfig::load().await;
        crate::trace::init_tracer(cfg);
        let service = get_service(cfg);
        let mut ctx = Context::init(cfg);

        // create test user
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let mut res = TestClient::post("http://localhost:3000/auth/signup")
            .json(&NewUser {
                name,
                email,
                password: "1234".to_string(),
            })
            .send(&service)
            .await;
        let body = res.take_json::<SignupRespBody>().await.unwrap();

        // issue the token and set the context
        let token = issue_user_token(&ctx, &body.user).unwrap();
        ctx.user = Some(body.user);

        // Run the test
        let (service, _ctx, token) = f(service, ctx, token).await;

        // cleanup
        let res = TestClient::delete("http://localhost:3000/auth/me")
            .add_header(AUTHORIZATION, format!("Bearer {token}"), true)
            .send(&service)
            .await;
        if !res.status_code.unwrap().is_success() {
            panic!("failed to delete test user");
        }
    }

    #[tokio::test]
    async fn test_login() {
        run_test(|service, ctx, token| async {
            let user = ctx.user.as_ref().unwrap().clone();
            let res = TestClient::post("http://localhost:3000/auth/login")
                .json(&LoginReqBody {
                    email: user.email,
                    password: "1234".to_string(),
                })
                .send(&service)
                .await;
            assert_eq!(res.status_code.unwrap(), StatusCode::OK);
            (service, ctx, token)
        })
        .await;
    }

    #[tokio::test]
    async fn test_me_get() {
        run_test(|service, ctx, token| async {
            let res = TestClient::get("http://localhost:3000/auth/me")
                .add_header(AUTHORIZATION, format!("Bearer {token}"), true)
                .send(&service)
                .await;
            assert_eq!(res.status_code.unwrap(), StatusCode::OK);
            (service, ctx, token)
        })
        .await;
    }

    #[tokio::test]
    async fn test_me_update() {
        run_test(|service, ctx, token| async {
            let res = TestClient::patch("http://localhost:3000/auth/me")
                .add_header(AUTHORIZATION, format!("Bearer {token}"), true)
                .json(&UserFields {
                    id: None,
                    name: Some("new Name".to_string()),
                    email: None,
                    password: None,
                })
                .send(&service)
                .await;
            assert_eq!(res.status_code.unwrap(), StatusCode::OK);
            (service, ctx, token)
        })
        .await;
    }
}
