//! Databases

pub mod user;

/// Database error
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Internal error
    #[error("internal error: {message}")]
    Internal {
        /// Error message
        message: String,
    },
}

impl From<deadpool_postgres::PoolError> for DbError {
    fn from(value: deadpool_postgres::PoolError) -> Self {
        DbError::Internal {
            message: value.to_string(),
        }
    }
}

impl From<tokio_postgres::Error> for DbError {
    fn from(value: tokio_postgres::Error) -> Self {
        DbError::Internal {
            message: value.to_string(),
        }
    }
}
