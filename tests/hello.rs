//! Client tests

#[tokio::test]
async fn test_hello() {
    let url = hyper::Uri::from_static("http://localhost:3000");
    let client = hyper::Client::new();

    let resp = client.get(url).await.unwrap();
    assert_eq!(resp.status(), hyper::StatusCode::OK)
}
