use std::fmt;
use std::error::Error;

/// The standard error type for rullst-orm, shielding users from internal dependency errors.
#[derive(Debug)]
pub enum RullstError {
    /// A record was not found in the database.
    RecordNotFound,
    /// A general database or query error.
    DatabaseError(String),
    /// A serialization or deserialization error (e.g., JSON).
    SerializationError(String),
    /// A cache or event-related error.
    CacheError(String),
    /// Other internal errors.
    Internal(String),
}

impl fmt::Display for RullstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RullstError::RecordNotFound => write!(f, "Record not found"),
            RullstError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            RullstError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            RullstError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            RullstError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl Error for RullstError {}

impl From<sqlx::Error> for RullstError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => RullstError::RecordNotFound,
            _ => RullstError::DatabaseError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for RullstError {
    fn from(err: serde_json::Error) -> Self {
        RullstError::SerializationError(err.to_string())
    }
}

#[cfg(feature = "redis")]
impl From<redis::RedisError> for RullstError {
    fn from(err: redis::RedisError) -> Self {
        RullstError::CacheError(err.to_string())
    }
}
