//! Summary tests

use crate::common::{setup, teardown};

mod common;

#[tokio::test]
async fn test_feeds() {
    let (client, _user, _) = setup().await;

    let urls = vec![
        "https://techcrunch.com/2023/07/11/outverse-wants-to-build-a-full-stack-community-platform-for-software-companies/",
        "https://www.suse.com/news/SUSE-Preserves-Choice-in-Enterprise-Linux/",
        "https://hackaday.com/2023/07/11/soviet-era-pong-console-is-easy-to-repair/"
    ];
    let summaries = client.summarize(&urls).await.unwrap();
    assert_eq!(summaries.len(), 3);
    println!("{summaries:?}");

    teardown(client).await;
}
