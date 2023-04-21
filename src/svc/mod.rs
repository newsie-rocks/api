//! Services

use self::auth::User;

pub mod auth;

/// Service context
#[derive(Debug)]
pub struct Context {
    /// Auth secret
    pub auth_secret: String,
    /// DB pool
    pub db_pool: deadpool_postgres::Pool,
    /// User
    pub user: Option<User>,
}
