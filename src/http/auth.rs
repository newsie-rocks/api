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

#[cfg(test)]
mod tests {
    use fake::{
        faker::{internet::en::FreeEmail, name::en::Name},
        Fake,
    };
    use salvo::{
        hyper::header::AUTHORIZATION,
        test::{ResponseExt, TestClient},
        Service,
    };
    use tokio::sync::OnceCell;

    use crate::{
        config::AppConfig,
        http::get_router,
        svc::auth::{NewUser, User},
    };

    use super::*;

    /// New user for tests
    static NEW_USER: OnceCell<(Service, User, String)> = OnceCell::const_new();

    /// Initializes the tests
    async fn init_tests() -> &'static (Service, User, String) {
        NEW_USER
            .get_or_init(|| async {
                // init tracer
                let cfg = AppConfig::load().await;
                crate::trace::init_tracer(cfg);

                let router = get_router(cfg);
                let service = Service::new(router);

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
                (service, body.user, body.token)
            })
            .await
    }

    #[tokio::test]
    async fn test_login() {
        let (service, user, _) = init_tests().await;
        let res = TestClient::post("http://localhost:3000/auth/login")
            .json(&LoginReqBody {
                email: user.email.clone(),
                password: "1234".to_string(),
            })
            .send(service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_me_get() {
        let (service, _, token) = init_tests().await;
        let res = TestClient::get("http://localhost:3000/auth/me")
            .add_header(AUTHORIZATION, format!("Bearer {token}"), true)
            .send(service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_me_update() {
        let (service, _, token) = init_tests().await;
        let mut res = TestClient::patch("http://localhost:3000/auth/me")
            .add_header(AUTHORIZATION, format!("Bearer {token}"), true)
            .json(&UserFields {
                id: None,
                name: Some("new Name".to_string()),
                email: None,
                password: None,
            })
            .send(service)
            .await;

        // BUG: when running all the auth tests, there is a 500 code on the update test
        // It is very random
        // it seems to be related to a connection closed error
        // it seems related to the fact that tests use different runtime, so the shared connection pool
        // is closed when the test ends
        // see: https://github.com/tokio-rs/tokio/issues/2374
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);

        let body = res.take_json::<GetUserRespBody>().await.unwrap();
        assert_eq!(body.user.name, "new Name".to_string());
    }
}
