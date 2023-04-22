//! Auth tests

#[tokio::test]
async fn test_signup() {
    let url = hyper::Uri::from_static("http://localhost:3000/auth/signup");
    let client = hyper::Client::new();

    let req_body = hyper::Body::from(
        r#"{
            "name": "nico",
            "email": "nla@melix.net",
            "password": "1234"
        }"#,
    );
    let req = hyper::Request::builder().uri(url).body(req_body).unwrap();
    let res: hyper::Response<hyper::Body> = client.request(req).await.unwrap();
    assert_eq!(res.status(), 201);
}
