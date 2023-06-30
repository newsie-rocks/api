//! Generates the OpenAPI documentation

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    api::http::get_root,
    api::http::healthcheck,
    //
))]
struct ApiDoc;

fn main() {
    println!("{}", ApiDoc::openapi().to_yaml().unwrap());
}
