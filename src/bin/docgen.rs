//! Generates the OpenAPI documentation

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    security((), ("bearer" = [])),
    paths(
        api::http::get_root,
        api::http::healthcheck,
        api::http::auth::signup,
        api::http::auth::login,
        api::http::auth::get_user,
    ),
    components(schemas(
        api::http::auth::SignupRespBody,
        api::http::auth::LoginReqBody,
        api::http::auth::LoginRespBody,
        api::http::auth::GetUserRespBody,
        api::svc::auth::User,
        api::svc::auth::NewUser,
    ))
)]
struct ApiDoc;

fn main() {
    println!("{}", ApiDoc::openapi().to_yaml().unwrap());
}
