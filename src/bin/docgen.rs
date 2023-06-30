//! Generates the OpenAPI documentation

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::http::get_root,
        api::http::healthcheck,
        api::http::auth::signup,
        //
    ), 
    components(
        schemas(
            api::http::auth::SignupReqBody,
            api::http::auth::SignupRespBody,
            api::svc::auth::User,
        )
    ))]
struct ApiDoc;

fn main() {
    println!("{}", ApiDoc::openapi().to_yaml().unwrap());
}
