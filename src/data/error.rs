//! Db errors

/// Store error
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Internal error
    #[error("internal error: {message}")]
    Internal {
        /// Error message
        message: String,
    },
}

impl From<deadpool_postgres::PoolError> for StoreError {
    fn from(value: deadpool_postgres::PoolError) -> Self {
        StoreError::Internal {
            message: value.to_string(),
        }
    }
}

impl From<tokio_postgres::Error> for StoreError {
    fn from(value: tokio_postgres::Error) -> Self {
        StoreError::Internal {
            message: value.to_string(),
        }
    }
}
