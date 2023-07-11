//! User tests

use newsie_client::UserUpdate;

use crate::common::{setup, teardown};

mod common;

#[tokio::test]
async fn test_login() {
    let (mut client, user, password) = setup().await;
    let res = client.login(&user.email, &password).await.unwrap();
    assert_eq!(res.user.email, user.email);
    teardown(client).await;
}

#[tokio::test]
async fn test_get_user() {
    let (client, user, _) = setup().await;
    let res = client.me().await.unwrap();
    assert_eq!(res.user.email, user.email);
    teardown(client).await;
}

#[tokio::test]
async fn test_update_user() {
    let (client, _, _) = setup().await;
    let res = client
        .update_me(UserUpdate {
            name: Some("new_name".to_string()),
            email: None,
            password: None,
        })
        .await
        .unwrap();
    assert_eq!(res.name, "new_name".to_string());
    teardown(client).await;
}
