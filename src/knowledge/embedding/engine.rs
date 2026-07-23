//! Embedding engine trait and implementations.

use crate::knowledge::embedding::cache::EmbeddingCache;
use crate::knowledge::embedding::model::TfidfEmbeddingModel;
use crate::knowledge::embedding::vector::DenseVector;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Result type for embedding operations.
pub type EmbeddingResult<T> = Result<T, EmbeddingError>;

/// Error type for embedding operations.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Embedding model not fitted: {0}")]
    ModelNotFitted(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Embedding error: {0}")]
    EmbeddingFailed(String),

    #[error("Lock poisoned: {0}")]
    LockError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

/// Result of embedding operation with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingOutput {
    pub vector: DenseVector,
    pub dimension: usize,
    pub normalized: bool,
}

/// Trait defining the interface for embedding engines.
///
/// Implementations can provide different backends (local, ONNX, remote, etc.)
/// without changing the consumer code.
pub trait EmbeddingEngine: Send + Sync {
    /// Embed a single text input.
    fn embed_text(&self, text: &str) -> EmbeddingResult<DenseVector>;

    /// Embed a document (longer text).
    fn embed_document(&self, document: &str) -> EmbeddingResult<DenseVector>;

    /// Embed multiple text chunks.
    fn embed_chunks(&self, chunks: &[&str]) -> EmbeddingResult<Vec<DenseVector>>;

    /// Embed a query (may be optimized differently from documents).
    fn embed_query(&self, query: &str) -> EmbeddingResult<DenseVector>;

    /// Get the embedding dimension.
    fn dimension(&self) -> usize;

    /// Normalize a vector in-place.
    fn normalize(&self, vector: &mut DenseVector) {
        vector.normalize_inplace();
    }

    /// Check if the engine is ready for use.
    fn is_ready(&self) -> bool;
}

/// A production-grade local embedding engine using TF-IDF with caching.
///
/// This implementation:
/// - Uses TF-IDF for deterministic, reproducible embeddings
/// - Caches embeddings with BLAKE3 content hashing
/// - Normalizes all output vectors
/// - Is thread-safe with Arc<Mutex<>>
pub struct LocalEmbeddingEngine {
    model: Arc<Mutex<TfidfEmbeddingModel>>,
    cache: Arc<Mutex<EmbeddingCache>>,
}

impl LocalEmbeddingEngine {
    /// Create a new local embedding engine with default configuration.
    pub fn new() -> Self {
        Self {
            model: Arc::new(Mutex::new(TfidfEmbeddingModel::new())),
            cache: Arc::new(Mutex::new(EmbeddingCache::new())),
        }
    }

    /// Create a new engine and fit it with documents.
    pub fn with_documents(documents: &[&str]) -> EmbeddingResult<Self> {
        let mut model = TfidfEmbeddingModel::new();
        model
            .fit(documents)
            .map_err(EmbeddingError::ModelNotFitted)?;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            cache: Arc::new(Mutex::new(EmbeddingCache::new())),
        })
    }

    /// Persist the fitted TF-IDF model to a file (JSON).
    pub fn persist_model(&self, path: &str) -> EmbeddingResult<()> {
        let model = self
            .model
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;
        let data =
            serde_json::to_string(&*model).map_err(|e| EmbeddingError::IoError(e.to_string()))?;
        std::fs::write(path, data).map_err(|e| EmbeddingError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Load a persisted TF-IDF model from a file and replace the current model.
    pub fn load_model(&self, path: &str) -> EmbeddingResult<()> {
        let data =
            std::fs::read_to_string(path).map_err(|e| EmbeddingError::IoError(e.to_string()))?;
        let model: TfidfEmbeddingModel =
            serde_json::from_str(&data).map_err(|e| EmbeddingError::IoError(e.to_string()))?;
        let mut guard = self
            .model
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;
        *guard = model;
        Ok(())
    }

    /// Rebuild the internal model from provided documents (full re-fit).
    pub fn rebuild_model(&self, documents: &[&str]) -> EmbeddingResult<()> {
        if documents.is_empty() {
            return Err(EmbeddingError::InvalidInput("documents empty".to_string()));
        }
        let mut new_model = TfidfEmbeddingModel::new();
        new_model
            .fit(documents)
            .map_err(EmbeddingError::EmbeddingFailed)?;
        let mut guard = self
            .model
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;
        *guard = new_model;
        Ok(())
    }

    /// Replace/Update the model using the provided documents (convenience for incremental update).
    pub fn update_model(&self, documents: &[&str]) -> EmbeddingResult<()> {
        // For now perform a full rebuild using the provided documents.
        self.rebuild_model(documents)
    }

    /// Return an engine version identifier.
    pub fn version(&self) -> &'static str {
        "tfidf-1.0"
    }

    /// Enable or disable caching.
    pub fn set_cache_enabled(&self, enabled: bool) -> EmbeddingResult<()> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;
        cache.set_enabled(enabled);
        Ok(())
    }

    /// Get cache statistics.
    pub fn cache_statistics(
        &self,
    ) -> EmbeddingResult<crate::knowledge::embedding::cache::EmbeddingCacheStats> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;
        Ok(cache.statistics())
    }

    /// Persist cache to disk.
    pub fn persist_cache(&self, path: &str) -> EmbeddingResult<()> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;
        cache.persist(path).map_err(EmbeddingError::IoError)
    }

    /// Load cache from disk.
    pub fn load_cache(&self, path: &str) -> EmbeddingResult<usize> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;
        cache.load(path).map_err(EmbeddingError::IoError)
    }
}

impl Default for LocalEmbeddingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingEngine for LocalEmbeddingEngine {
    fn embed_text(&self, text: &str) -> EmbeddingResult<DenseVector> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text".to_string()));
        }

        let model = self
            .model
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;

        let cache = self
            .cache
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;

        let result = cache.get_or_insert(text, || model.embed(text));
        Ok(result)
    }

    fn embed_document(&self, document: &str) -> EmbeddingResult<DenseVector> {
        self.embed_text(document)
    }

    fn embed_chunks(&self, chunks: &[&str]) -> EmbeddingResult<Vec<DenseVector>> {
        let model = self
            .model
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;

        let cache = self
            .cache
            .lock()
            .map_err(|e| EmbeddingError::LockError(e.to_string()))?;

        let results = chunks
            .iter()
            .map(|chunk| cache.get_or_insert(chunk, || model.embed(chunk)))
            .collect();

        Ok(results)
    }

    fn embed_query(&self, query: &str) -> EmbeddingResult<DenseVector> {
        self.embed_text(query)
    }

    fn dimension(&self) -> usize {
        let model = self.model.lock().ok().map(|m| m.dimension()).unwrap_or(768);
        model
    }

    fn is_ready(&self) -> bool {
        self.model.lock().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_engine_creation() {
        let engine = LocalEmbeddingEngine::new();
        assert!(engine.is_ready());
        assert_eq!(engine.dimension(), 768);
    }

    #[test]
    fn test_local_engine_with_documents() {
        let docs = vec!["hello world", "test document"];
        let engine = LocalEmbeddingEngine::with_documents(&docs).unwrap();
        let embedding = engine.embed_text("hello").unwrap();
        assert_eq!(embedding.dimension(), 768);
    }

    #[test]
    fn test_embed_chunks() {
        let docs = vec!["hello world", "test document"];
        let engine = LocalEmbeddingEngine::with_documents(&docs).unwrap();
        let chunks = vec!["hello", "world"];
        let embeddings = engine.embed_chunks(&chunks).unwrap();
        assert_eq!(embeddings.len(), 2);
    }

    #[test]
    fn test_cache_hit() {
        let docs = vec!["test"];
        let engine = LocalEmbeddingEngine::with_documents(&docs).unwrap();

        let emb1 = engine.embed_text("test").unwrap();
        let emb2 = engine.embed_text("test").unwrap();
        assert_eq!(emb1.data(), emb2.data());
    }

    #[test]
    fn test_cache_statistics() {
        let docs = vec!["test"];
        let engine = LocalEmbeddingEngine::with_documents(&docs).unwrap();

        engine.embed_text("hello").unwrap();
        engine.embed_text("hello").unwrap();

        let stats = engine.cache_statistics().unwrap();
        assert_eq!(stats.hit_count, 1);
    }
}
