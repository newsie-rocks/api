//! Client tests

#[tokio::test]
async fn test_root() {
    let client = hyper::Client::new();
    let uri = hyper::Uri::from_static("http://localhost:3000");

    let res = client.get(uri).await.unwrap();
    assert_eq!(res.status(), hyper::StatusCode::OK)
}

#[tokio::test]
async fn test_healthcheck() {
    let client = hyper::Client::new();
    let uri = hyper::Uri::from_static("http://localhost:3000/up");

    let res = client.get(uri).await.unwrap();
    assert_eq!(res.status(), hyper::StatusCode::OK)
}
