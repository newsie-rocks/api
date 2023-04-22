//! Services

use std::sync::Arc;

use self::auth::User;

pub mod auth;

/// Service context
#[derive(Debug)]
pub struct Context {
    /// Auth secret
    pub auth_secret: String,
    /// DB pool
    pub db_pool: Arc<deadpool_postgres::Pool>,
    /// User
    pub user: Option<User>,
}
