//! Feeds

use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User feed
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserFeed {
    /// ID
    pub id: Uuid,
    /// User id
    pub user_id: Uuid,
    /// Feed url
    pub url: String,
    /// Feed name
    pub name: Option<String>,
    // .. add other user/feed settings
}

/// User article
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserArticle {
    /// ID
    pub id: Uuid,
    /// User feed id
    pub user_feed_id: Uuid,
    /// Article url
    pub url: String,
    /// Is read
    pub read: bool,
    /// Is starred
    pub starred: bool,
}

/// Feed service error
#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    /// Unauthenticated
    #[error("Unauthenticated: {message}")]
    Unauthenticated {
        /// Message
        message: String,
    },
    /// Internal error
    #[error("{message}")]
    Internal {
        /// Message
        message: String,
    },
}

/// Fetches all user feeds
pub fn get_user_feeds() -> Vec<UserFeed> {
    todo!()
}

/// Adds a user feed
pub fn add_user_feed() {
    todo!()
}

/// Removes a user feed
pub fn remove_user_feed() {
    todo!()
}

/// Updates a feed
pub fn update_user_feed() {
    todo!()
}
