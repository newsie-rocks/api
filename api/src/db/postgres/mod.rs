//! Postgres DB

use crate::error::Error;

pub mod feed;
pub mod user;

/// Postgres DB
#[derive(Debug, Clone)]
pub struct PostgresDb {
    /// Postgres pool
    pool: deadpool_postgres::Pool,
}

impl PostgresDb {
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
}
