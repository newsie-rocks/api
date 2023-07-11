//! Postgres DB

use crate::error::Error;

pub mod feed;
pub mod summary;
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
        self.init_custom_types().await?;
        self.create_table_users().await?;
        self.create_table_feeds().await?;
        self.create_table_summaries().await?;
        Ok(())
    }

    /// Initializes the PG vector extension
    async fn init_pgvector(&self) -> Result<(), Error> {
        let client = self.client().await?;
        Ok(client
            .batch_execute("CREATE EXTENSION IF NOT EXISTS vector;")
            .await?)
    }

    /// Initializes custom types (eg enums, ...)
    async fn init_custom_types(&self) -> Result<(), Error> {
        let client = self.client().await?;
        Ok(client
            .batch_execute(
                "
                CREATE TYPE subscription AS ENUM (
                    'FREE',
                    'MID'
                )
                ",
            )
            .await?)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::config::AppConfig;

    use super::*;

    /// Initializes the user store
    fn init_db() -> PostgresClient {
        let cfg = AppConfig::load();
        PostgresClient::new(cfg.postgres.new_pool())
    }

    #[ignore]
    #[tokio::test]
    async fn test_init_schema() {
        let client = init_db();
        client.init_schema().await.unwrap();
    }
}
