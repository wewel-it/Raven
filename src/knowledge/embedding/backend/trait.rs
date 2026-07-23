//! Core trait for embedding backends.
//!
//! This module defines the `EmbeddingBackend` trait that all embedding
//! implementations must satisfy.

use crate::knowledge::embedding::vector::DenseVector;
use async_trait::async_trait;
use std::sync::Arc;

/// Result type for backend operations.
pub type BackendResult<T> = Result<T, BackendError>;

/// Error type for backend operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BackendError {
    #[error("model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("model load failed: {0}")]
    ModelLoadFailed(String),

    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("embedding failed: {0}")]
    EmbeddingFailed(String),

    #[error("batch embedding failed: {0}")]
    BatchEmbeddingFailed(String),

    #[error("io error: {0}")]
    IoError(String),

    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("timeout: {0}")]
    Timeout(String),
}

/// Trait for embedding backends.
///
/// This trait defines the interface that all embedding backends must implement.
/// Implementations can be local (TF-IDF, ONNX) or remote (API-based).
#[async_trait]
pub trait EmbeddingBackend: Send + Sync {
    /// Embed a single text.
    async fn embed_text(&self, text: &str) -> BackendResult<DenseVector>;

    /// Embed multiple texts in a batch.
    ///
    /// This method should be more efficient than calling `embed_text` multiple times.
    async fn embed_batch(&self, texts: &[&str]) -> BackendResult<Vec<DenseVector>>;

    /// Get the embedding dimension.
    fn embedding_dimension(&self) -> usize;

    /// Get the model name/identifier.
    fn model_name(&self) -> &str;

    /// Normalize a vector.
    fn normalize(&self, vector: &mut DenseVector) {
        vector.normalize_inplace();
    }

    /// Check if the backend supports batch embedding.
    fn supports_batch(&self) -> bool {
        true
    }

    /// Check if the backend supports GPU acceleration.
    fn supports_gpu(&self) -> bool {
        false
    }

    /// Check if the backend supports CPU computation.
    fn supports_cpu(&self) -> bool {
        true
    }

    /// Check if the backend is ready for use.
    fn is_ready(&self) -> bool;

    /// Load the model (if needed).
    async fn load(&self) -> BackendResult<()> {
        Ok(())
    }

    /// Unload the model (if needed).
    async fn unload(&self) -> BackendResult<()> {
        Ok(())
    }

    /// Get backend metadata.
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            model_name: self.model_name().to_string(),
            dimension: self.embedding_dimension(),
            supports_batch: self.supports_batch(),
            supports_gpu: self.supports_gpu(),
            supports_cpu: self.supports_cpu(),
        }
    }
}

/// Metadata about an embedding backend.
#[derive(Debug, Clone)]
pub struct BackendMetadata {
    pub model_name: String,
    pub dimension: usize,
    pub supports_batch: bool,
    pub supports_gpu: bool,
    pub supports_cpu: bool,
}
