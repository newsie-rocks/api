//! Shared utilities

use fake::{
    faker::{
        internet::en::{FreeEmail, Password},
        name::en::Name,
    },
    Fake,
};
use newsie_client::{Client, NewUser, User};

/// Setup the test
pub async fn setup() -> (Client, User, String) {
    let mut client = Client::new("http://localhost:3000");
    let name: String = Name().fake();
    let email: String = FreeEmail().fake();
    let password: String = Password(10..20).fake();
    let res = client
        .signup(NewUser {
            name,
            email,
            password: password.clone(),
        })
        .await
        .unwrap();
    let user = res.user;

    (client, user, password)
}

/// Teardown a test
pub async fn teardown(mut client: Client) {
    client.delete_me().await.unwrap();
}
