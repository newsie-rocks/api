//! Feed service

use uuid::Uuid;

use crate::{
    db::postgres::PostgresClient,
    error::Error,
    mdl::{Feed, FeedUpdate},
};

/// Feed service
#[derive(Debug, Clone)]
pub struct FeedService {
    /// Postgres db
    pub db: PostgresClient,
}

impl FeedService {
    /// Creates a new service instance
    pub fn new(postgres_client: PostgresClient) -> Self {
        Self {
            db: postgres_client,
        }
    }
}

impl FeedService {
    /// Gets all the user feeds
    pub async fn get_feeds(&self, user_id: Uuid) -> Result<Vec<Feed>, Error> {
        self.db.read_user_feeds(user_id).await
    }

    /// Sync the user feeds
    pub async fn sync_feeds(
        &self,
        user_id: Uuid,
        feeds: Vec<FeedUpdate>,
    ) -> Result<Vec<Feed>, Error> {
        self.db.sync_user_feeds(user_id, feeds).await
    }
}
