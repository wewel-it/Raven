use crate::error::RavenError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("lock poisoned: {0}")]
    Poison(String),
    #[error("persistence error: {0}")]
    Persistence(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid memory request: {0}")]
    InvalidRequest(String),
}

impl From<crate::memory::MemoryService> for MemoryError {
    fn from(_err: crate::memory::MemoryService) -> Self {
        MemoryError::InvalidRequest("conversion from MemoryService not supported".to_string())
    }
}
