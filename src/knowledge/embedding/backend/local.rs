//! Local Hash Embedding backend (fallback).
//!
//! This module provides a lightweight fallback embedding using:
//! - Configurable dimensions (default 256)
//! - Deterministic content-based hashing
//! - No external dependencies
//! - Minimal memory footprint

use super::trait_impl::{BackendError, BackendResult, EmbeddingBackend};
use crate::knowledge::embedding::vector::DenseVector;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Local hash embedding backend.
///
/// This is a lightweight fallback embedding that:
/// - Uses BLAKE3 hashing for deterministic embeddings
/// - Supports configurable dimensions
/// - Has minimal memory overhead
/// - Is suitable for development and testing
pub struct LocalHashBackend {
    dimension: usize,
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
}

impl LocalHashBackend {
    /// Create a new local hash backend with default dimension (256).
    pub fn new() -> Self {
        Self::with_dimension(256)
    }

    /// Create a new local hash backend with specified dimension.
    pub fn with_dimension(dimension: usize) -> Self {
        if dimension < 64 || dimension > 4096 {
            // Clamp to valid range if needed
            let clamped = dimension.max(64).min(4096);
            return Self {
                dimension: clamped,
                cache: Arc::new(Mutex::new(HashMap::new())),
            };
        }

        Self {
            dimension,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate deterministic embedding from content using hash.
    fn generate_embedding(&self, text: &str) -> DenseVector {
        let hash = blake3::hash(text.as_bytes());
        let hash_bytes = hash.as_bytes();

        // Use the hash bytes to generate the embedding
        let mut embedding = Vec::with_capacity(self.dimension);

        for i in 0..self.dimension {
            // Cycle through hash bytes
            let byte_index = (i * 8) % hash_bytes.len();
            let byte_value = hash_bytes[byte_index] as f32;

            // Convert to float in range [-1.0, 1.0]
            let float_val = (byte_value / 255.0) * 2.0 - 1.0;
            embedding.push(float_val);
        }

        let mut vector = DenseVector::new(embedding);
        vector.normalize_inplace();
        vector
    }
}

#[async_trait]
impl EmbeddingBackend for LocalHashBackend {
    async fn embed_text(&self, text: &str) -> BackendResult<DenseVector> {
        if text.is_empty() {
            return Err(BackendError::InvalidInput(
                "text cannot be empty".to_string(),
            ));
        }

        // Check cache
        {
            let cache = self
                .cache
                .lock()
                .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?;
            if let Some(cached) = cache.get(text) {
                return Ok(cached.clone());
            }
        }

        let embedding = self.generate_embedding(text);

        // Cache result
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?;
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> BackendResult<Vec<DenseVector>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            if text.is_empty() {
                return Err(BackendError::InvalidInput(
                    "text cannot be empty".to_string(),
                ));
            }
            let embedding = self.embed_text(text).await?;
            results.push(embedding);
        }

        Ok(results)
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "local-hash"
    }

    fn is_ready(&self) -> bool {
        true
    }
}

impl Default for LocalHashBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_hash_default() {
        let backend = LocalHashBackend::new();
        let embedding = backend.embed_text("Hello, world!").await.unwrap();
        assert_eq!(embedding.dimension(), 256);
    }

    #[tokio::test]
    async fn test_local_hash_custom_dimension() {
        let backend = LocalHashBackend::with_dimension(512);
        let embedding = backend.embed_text("Hello, world!").await.unwrap();
        assert_eq!(embedding.dimension(), 512);
    }

    #[tokio::test]
    async fn test_local_hash_deterministic() {
        let backend = LocalHashBackend::new();
        let emb1 = backend.embed_text("test").await.unwrap();
        let emb2 = backend.embed_text("test").await.unwrap();
        assert_eq!(emb1.data(), emb2.data());
    }

    #[tokio::test]
    async fn test_local_hash_batch() {
        let backend = LocalHashBackend::new();
        let texts = vec!["hello", "world", "test"];
        let embeddings = backend.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);
        for emb in embeddings {
            assert_eq!(emb.dimension(), 256);
        }
    }

    #[tokio::test]
    async fn test_local_hash_dimension_clamping() {
        let backend1 = LocalHashBackend::with_dimension(32);
        assert_eq!(backend1.embedding_dimension(), 64); // min

        let backend2 = LocalHashBackend::with_dimension(8192);
        assert_eq!(backend2.embedding_dimension(), 4096); // max
    }
}
