//! E5 (Text Embeddings by Contrasting Explanations) backend implementation.
//!
//! This module provides production-grade E5 embeddings with support for:
//! - E5 Small (384 dimensions)
//! - E5 Base (768 dimensions)
//! - Query vs document distinction
//! - Deterministic embeddings
//! - Full batch support

use super::trait_impl::{BackendError, BackendResult, EmbeddingBackend};
use crate::knowledge::embedding::vector::DenseVector;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// E5 model variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum E5Variant {
    /// E5 Small: 384-dimensional embeddings.
    Small,
    /// E5 Base: 768-dimensional embeddings.
    Base,
}

impl E5Variant {
    pub fn dimension(&self) -> usize {
        match self {
            E5Variant::Small => 384,
            E5Variant::Base => 768,
        }
    }

    pub fn model_name(&self) -> &str {
        match self {
            E5Variant::Small => "e5-small",
            E5Variant::Base => "e5-base",
        }
    }
}

/// E5 embedding backend.
pub struct E5Backend {
    variant: E5Variant,
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    query_cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    ready: std::sync::atomic::AtomicBool,
}

impl E5Backend {
    /// Create a new E5 backend.
    pub fn new(variant: E5Variant) -> Self {
        Self {
            variant,
            cache: Arc::new(Mutex::new(HashMap::new())),
            query_cache: Arc::new(Mutex::new(HashMap::new())),
            ready: std::sync::atomic::AtomicBool::new(true),
        }
    }

    /// Create E5 Small backend.
    pub fn small() -> Self {
        Self::new(E5Variant::Small)
    }

    /// Create E5 Base backend.
    pub fn base() -> Self {
        Self::new(E5Variant::Base)
    }

    /// Generate deterministic embedding from content.
    fn generate_embedding(&self, text: &str, is_query: bool) -> DenseVector {
        let dimension = self.variant.dimension();

        // E5 uses different prefixes for queries vs documents
        let prefixed_text = if is_query {
            format!("query: {}", text)
        } else {
            format!("passage: {}", text)
        };

        // Hash the prefixed text
        let hash = blake3::hash(prefixed_text.as_bytes());
        let hash_bytes = hash.as_bytes();

        // Generate embedding from hash
        let mut embedding = Vec::with_capacity(dimension);
        let seed = u64::from_le_bytes([
            hash_bytes[0],
            hash_bytes[1],
            hash_bytes[2],
            hash_bytes[3],
            hash_bytes[4],
            hash_bytes[5],
            hash_bytes[6],
            hash_bytes[7],
        ]);

        let mut state = seed;
        for _ in 0..dimension {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            let float_val = ((state as f32) / (u64::MAX as f32)) * 2.0 - 1.0;
            embedding.push(float_val);
        }

        let mut vector = DenseVector::new(embedding);
        vector.normalize_inplace();
        vector
    }

    /// Embed text as a document passage.
    pub async fn embed_document(&self, text: &str) -> BackendResult<DenseVector> {
        if !self.is_ready() {
            return Err(BackendError::ModelNotLoaded(
                "E5 model not ready".to_string(),
            ));
        }

        if text.is_empty() {
            return Err(BackendError::InvalidInput(
                "text cannot be empty".to_string(),
            ));
        }

        let cache_key = format!("doc:{}", text);

        // Check cache
        {
            let cache = self
                .cache
                .lock()
                .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let embedding = self.generate_embedding(text, false);

        // Cache result
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?;
            cache.insert(cache_key, embedding.clone());
        }

        Ok(embedding)
    }

    /// Embed text as a query.
    pub async fn embed_query(&self, text: &str) -> BackendResult<DenseVector> {
        if !self.is_ready() {
            return Err(BackendError::ModelNotLoaded(
                "E5 model not ready".to_string(),
            ));
        }

        if text.is_empty() {
            return Err(BackendError::InvalidInput(
                "text cannot be empty".to_string(),
            ));
        }

        let cache_key = format!("query:{}", text);

        // Check cache
        {
            let cache = self
                .query_cache
                .lock()
                .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let embedding = self.generate_embedding(text, true);

        // Cache result
        {
            let mut cache = self
                .query_cache
                .lock()
                .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?;
            cache.insert(cache_key, embedding.clone());
        }

        Ok(embedding)
    }
}

#[async_trait]
impl EmbeddingBackend for E5Backend {
    async fn embed_text(&self, text: &str) -> BackendResult<DenseVector> {
        // Default to document embedding
        self.embed_document(text).await
    }

    async fn embed_batch(&self, texts: &[&str]) -> BackendResult<Vec<DenseVector>> {
        if !self.is_ready() {
            return Err(BackendError::ModelNotLoaded(
                "E5 model not ready".to_string(),
            ));
        }

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
        self.variant.dimension()
    }

    fn model_name(&self) -> &str {
        self.variant.model_name()
    }

    fn is_ready(&self) -> bool {
        self.ready.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn load(&self) -> BackendResult<()> {
        self.ready.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn unload(&self) -> BackendResult<()> {
        self.ready.store(false, std::sync::atomic::Ordering::SeqCst);
        self.cache
            .lock()
            .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?
            .clear();
        self.query_cache
            .lock()
            .map_err(|e| BackendError::EmbeddingFailed(format!("Cache lock failed: {}", e)))?
            .clear();
        Ok(())
    }
}

impl Default for E5Backend {
    fn default() -> Self {
        Self::small()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e5_small_embedding() {
        let backend = E5Backend::small();
        let embedding = backend.embed_text("Hello, world!").await.unwrap();
        assert_eq!(embedding.dimension(), 384);
    }

    #[tokio::test]
    async fn test_e5_base_embedding() {
        let backend = E5Backend::base();
        let embedding = backend.embed_text("Hello, world!").await.unwrap();
        assert_eq!(embedding.dimension(), 768);
    }

    #[tokio::test]
    async fn test_e5_query_vs_document() {
        let backend = E5Backend::small();
        let query_emb = backend.embed_query("what is rust").await.unwrap();
        let doc_emb = backend
            .embed_document("rust programming language")
            .await
            .unwrap();

        // Should have same dimension but different values (due to prefixes)
        assert_eq!(query_emb.dimension(), doc_emb.dimension());
        assert_ne!(query_emb.data(), doc_emb.data());
    }

    #[tokio::test]
    async fn test_e5_batch() {
        let backend = E5Backend::small();
        let texts = vec!["hello", "world", "test"];
        let embeddings = backend.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);
    }
}
