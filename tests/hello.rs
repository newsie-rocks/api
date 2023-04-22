//! Client tests

#[tokio::test]
async fn test_hello() {
    let url = hyper::Uri::from_static("http://localhost:3000");
    let client = hyper::Client::new();

    let res = client.get(url).await.unwrap();
    assert_eq!(res.status(), hyper::StatusCode::OK)
}
