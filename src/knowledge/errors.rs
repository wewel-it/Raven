use crate::error::RavenError;
use thiserror::Error;

/// Result type for knowledge library operations.
pub type KnowledgeResult<T> = Result<T, KnowledgeError>;

#[derive(Error, Debug)]
pub enum KnowledgeError {
    #[error("invalid document: {0}")]
    InvalidDocument(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("hash error: {0}")]
    Hash(String),

    #[error("pipeline error: {0}")]
    Pipeline(String),

    #[error("runtime error: {0}")]
    Runtime(String),
}

impl From<std::io::Error> for KnowledgeError {
    fn from(err: std::io::Error) -> Self {
        KnowledgeError::Io(err.to_string())
    }
}

impl From<crate::knowledge::errors::KnowledgeError> for RavenError {
    fn from(err: crate::knowledge::errors::KnowledgeError) -> Self {
        RavenError::Unsupported(err.to_string())
    }
}
