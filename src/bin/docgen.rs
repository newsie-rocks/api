//! Generates the OpenAPI documentation

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    api::http::handle_base,
    //
))]
struct ApiDoc;

fn main() {
    println!("{}", ApiDoc::openapi().to_yaml().unwrap());
}
