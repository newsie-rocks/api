//! Services

use std::sync::Arc;

use qdrant_client::prelude::*;

use self::auth::User;

pub mod auth;
pub mod rss;

/// Service context
pub struct Context {
    /// Auth secret
    pub auth_secret: String,
    /// PostGres pool
    pub postgres_pool: deadpool_postgres::Pool,
    /// Qdrant client
    pub qdrant_client: Arc<QdrantClient>,
    /// User
    pub user: Option<User>,
}
