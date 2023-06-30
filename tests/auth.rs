//! Auth tests

use api::{http::auth::SignupRespBody, svc::auth::User};
use fake::{faker::internet::en::FreeEmail, Fake};
use tokio::sync::OnceCell;

/// Created user
static INIT_USER: OnceCell<(User, String)> = OnceCell::const_new();

/// Initializes a new user
async fn init_user() -> &'static (User, String) {
    INIT_USER
        .get_or_init(|| async {
            let client = hyper::Client::new();
            let uri = hyper::Uri::from_static("http://localhost:3000/auth/signup");

            let email: String = FreeEmail().fake();
            let req_body = hyper::Body::from(format!(
                r#"{{
                    "name": "test user",
                    "email": "{email}",
                    "password": "1234"
                }}"#
            ));
            let req = hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri(uri)
                .body(req_body)
                .unwrap();

            let res: hyper::Response<hyper::Body> = client.request(req).await.unwrap();
            let res_body = hyper::body::to_bytes(res.into_body()).await.unwrap();
            let res_parsed = serde_json::from_slice::<SignupRespBody>(&res_body).unwrap();
            (res_parsed.user, res_parsed.token)
        })
        .await
}

#[tokio::test]
async fn test_login() {
    let (user, token) = init_user().await;

    let client = hyper::Client::new();
    let uri = hyper::Uri::from_static("http://localhost:3000/auth/login");
    let req_body = hyper::Body::from(format!(
        r#"{{
            "email": "{email}",
            "password": "1234"
        }}"#,
        email = user.email
    ));
    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header("Authorization", format!("Bearer {}", token))
        .uri(uri)
        .body(req_body)
        .unwrap();

    let res: hyper::Response<hyper::Body> = client.request(req).await.unwrap();
    if !res.status().is_success() {
        let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let msg = String::from_utf8(body_bytes.to_vec()).unwrap();
        println!("ERR = {msg}");
    }
}
