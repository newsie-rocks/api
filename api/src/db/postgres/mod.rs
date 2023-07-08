//! Postgres DB

use crate::error::Error;

pub mod art;
pub mod feed;
pub mod user;
pub mod util;

/// Postgres DB
#[derive(Debug, Clone)]
pub struct PostgresClient {
    /// Postgres pool
    pool: deadpool_postgres::Pool,
}

impl PostgresClient {
    /// Creates a new instance
    pub fn new(postgres_pool: deadpool_postgres::Pool) -> Self {
        Self {
            pool: postgres_pool,
        }
    }

    /// Returns a postgres client instance
    async fn client(&self) -> Result<deadpool_postgres::Object, Error> {
        Ok(self.pool.get().await?)
    }

    /// Initializes the DB schema
    pub async fn init_schema(&self) -> Result<(), Error> {
        self.init_pgvector().await?;
        self.create_table_users().await?;
        self.create_table_feeds().await?;
        self.create_table_articles().await?;
        Ok(())
    }

    /// Initializes the PG vector extension
    async fn init_pgvector(&self) -> Result<(), Error> {
        let client = self.client().await?;
        Ok(client
            .batch_execute("CREATE EXTENSION IF NOT EXISTS vector;")
            .await?)
    }
}
