//! Services

use std::sync::Arc;

use crate::config::AppConfig;

use self::auth::User;

pub mod auth;
pub mod rss;

/// Service context
#[derive(Debug)]
pub struct Context {
    /// Configuration
    pub cfg: &'static AppConfig,
    /// DB pool
    pub db_pool: Arc<deadpool_postgres::Pool>,
    /// User
    pub user: Option<User>,
}
