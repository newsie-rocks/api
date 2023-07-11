//! Feed tests

use newsie_client::FeedUpdate;

use crate::common::{setup, teardown};

mod common;

#[tokio::test]
async fn test_feeds() {
    let (client, _user, _) = setup().await;

    let feeds = client.get_feeds().await.unwrap();
    assert_eq!(feeds.len(), 0);

    let my_feeds = vec![
        FeedUpdate {
            id: None,
            url: "http://www.google.com".to_string(),
            name: Some("Google".to_string()),
        },
        FeedUpdate {
            id: None,
            url: "http://www.google.com".to_string(),
            name: None,
        },
    ];
    let feeds = client.sync_feeds(&my_feeds).await.unwrap();
    assert_eq!(feeds.len(), 2);

    let feeds = client.sync_feeds(&[]).await.unwrap();
    assert_eq!(feeds.len(), 0);

    teardown(client).await;
}
