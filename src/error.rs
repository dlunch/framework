use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrameworkError {
    #[error("Database failure: {0}")]
    DatabaseError(String),
    #[error("Serialization failure")]
    SerializationError(String),
    #[error("Invalid event version, {0} expected, got {1}")]
    InvalidEventVersion(u32, u32),
    #[error("Concurrency error")]
    ConcurrencyError,
    #[error("Invalid query")]
    NoSuchReadModelStore,
}
