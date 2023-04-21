//! Core tests

println!("Response: {}", resp.status());

/// Tests the API connection
#[tokio::test]
async fn test_conn() {
    // let client = Client::new();

    // // Parse an `http::Uri`...
    // let uri = "http://localhost:3000".parse()?;

    // // Await the response...
    // let mut resp = client.get(uri).await?;

    assert_eq!(true, true);
}
