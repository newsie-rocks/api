//! Services

use std::sync::Arc;

use qdrant_client::prelude::*;

use crate::config::AppConfig;

use self::auth::User;

pub mod auth;
pub mod rss;

/// Service context
#[derive(Clone)]
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

impl Context {
    /// Initializes a new [Context]
    pub fn init(cfg: &AppConfig) -> Self {
        let auth_secret = cfg.auth.secret.clone();
        let postgres_pool = cfg.postgres.new_pool();
        let qdrant_client = Arc::new(cfg.qdrant.new_client());
        Context {
            auth_secret,
            postgres_pool,
            qdrant_client,
            user: None,
        }
    }
}
