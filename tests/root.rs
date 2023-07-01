use api::http::get_router;
use salvo::{prelude::*, test::TestClient};

#[tokio::test]
async fn test_root() {
    let router = get_router();
    let service = Service::new(router);
    let res = TestClient::get("http://localhost:3000")
        .send(&service)
        .await;
    assert_eq!(res.status_code.unwrap(), StatusCode::OK);
}

#[tokio::test]
async fn test_healthcheck() {
    let router = get_router();
    let service = Service::new(router);
    let res = TestClient::get("http://localhost:3000/up")
        .send(&service)
        .await;
    assert_eq!(res.status_code.unwrap(), StatusCode::OK);
}
