//! Feed service

use uuid::Uuid;

use crate::{
    db::postgres::PostgresDb,
    error::Error,
    mdl::{Feed, FeedUpdate},
};

/// Feed service
#[derive(Debug, Clone)]
pub struct FeedService {
    /// Postgres db
    pub db: PostgresDb,
}

impl FeedService {
    /// Creates a new service instance
    pub fn new(postgres_pool: deadpool_postgres::Pool) -> Self {
        let db = PostgresDb::new(postgres_pool);
        Self { db }
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
