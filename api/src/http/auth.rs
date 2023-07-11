//! Auth handlers

use cookie::Cookie;
use salvo::{oapi::extract::*, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    error::Error,
    http::ApiServices,
    mdl::{NewUser, SubscriptionUpdate, User, UserUpdate},
};

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
    body: JsonBody<NewUser>,
    res: &mut Response,
) -> Result<Json<SignupRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();

    let new_user = body.into_inner();
    let user = services.auth.create_user(new_user).await?;
    let token = services.auth.issue_token(&user)?;
    let auth_cookie = issue_auth_cookie(&token);

    res.status_code(StatusCode::CREATED);
    res.add_cookie(auth_cookie);
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
    pub token: String,
    /// User
    pub user: User,
}

/// Handles the login request
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn login(
    depot: &mut Depot,
    body: JsonBody<LoginReqBody>,
    res: &mut Response,
) -> Result<Json<LoginRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();

    let payload = body.into_inner();
    let user = services
        .auth
        .login(&payload.email, &payload.password)
        .await?;
    let token = services.auth.issue_token(&user)?;
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
    pub user: User,
}

/// Fetches the current user
#[endpoint(security(["bearerAuth" = []]))]
#[tracing::instrument(skip_all)]
pub async fn get_me(depot: &mut Depot) -> Result<Json<GetUserRespBody>, Error> {
    trace!("received request");
    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    Ok(Json(GetUserRespBody { user: user.clone() }))
}

/// Updates the current user
#[endpoint(security(["bearerAuth" = []]))]
#[tracing::instrument(skip_all)]
pub async fn update_me(
    depot: &mut Depot,
    body: JsonBody<UserUpdate>,
) -> Result<Json<GetUserRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();
    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    let user = services
        .auth
        .update_user(user.id, body.into_inner())
        .await?;

    Ok(Json(GetUserRespBody { user }))
}

/// Deletes a user
///
/// The ID is retrieved from the token
#[endpoint(security(["bearerAuth" = []]))]
#[tracing::instrument(skip_all)]
pub async fn delete_me(depot: &mut Depot) -> Result<(), Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();
    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    services.auth.delete_user(user.id).await?;

    Ok(())
}

/// Updates a subscription
#[endpoint(security(["bearerAuth" = []]))]
#[tracing::instrument(skip_all)]
pub async fn put_subscription(
    depot: &mut Depot,
    body: JsonBody<SubscriptionUpdate>,
    _res: &mut Response,
) -> Result<Json<GetUserRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();

    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    let subsc_update = body.into_inner();
    let user = services
        .auth
        .update_subscription(user.id, subsc_update)
        .await?;

    Ok(Json(GetUserRespBody { user }))
}

/// Authentication cookie key
pub const AUTH_COOKIE_NAME: &str = "newsie/auth_token";

/// Issues a new authentication cookie
pub fn issue_auth_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(AUTH_COOKIE_NAME, token.to_string())
        .http_only(true)
        // .domain("www.rust-lang.org")
        // .path("/")
        // .secure(true)
        .finish()
}

#[cfg(test)]
mod tests {
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
        config::AppConfig, db::postgres::PostgresClient, http::init_service, svc::auth::AuthService,
    };

    // Setup a test
    async fn setup() -> (Service, User, String) {
        // setup
        let cfg = AppConfig::load();
        crate::trace::init_tracer(&cfg);
        let service = init_service(&cfg).await;
        let postgres_client = PostgresClient::new(cfg.postgres.new_pool());
        let auth = AuthService::new(postgres_client, cfg.auth.secret.clone());

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
        let user = body.user;

        // issue the token
        let token = auth.issue_token(&user).unwrap();

        (service, user, token)
    }

    /// Teardown a test
    async fn teardown(service: Service, token: String) {
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
        let (service, user, token) = setup().await;
        let res = TestClient::post("http://localhost:3000/auth/login")
            .json(&LoginReqBody {
                email: user.email.clone(),
                password: "1234".to_string(),
            })
            .send(&service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);
        teardown(service, token).await;
    }

    #[tokio::test]
    async fn test_me_get() {
        let (service, _user, token) = setup().await;
        let res = TestClient::get("http://localhost:3000/auth/me")
            .add_header(AUTHORIZATION, format!("Bearer {token}"), true)
            .send(&service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);
        teardown(service, token).await;
    }

    #[tokio::test]
    async fn test_me_update() {
        let (service, _user, token) = setup().await;
        let res = TestClient::patch("http://localhost:3000/auth/me")
            .add_header(AUTHORIZATION, format!("Bearer {token}"), true)
            .json(&UserUpdate {
                name: Some("new Name".to_string()),
                email: None,
                password: None,
            })
            .send(&service)
            .await;
        assert_eq!(res.status_code.unwrap(), StatusCode::OK);
        teardown(service, token).await;
    }
}
