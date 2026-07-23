//! Production-grade embedding service.
//!
//! This module provides the main embedding service that integrates
//! backends, caching, batching, and metrics.

use super::backend::{BackendConfig, BackendRegistry, EmbeddingBackend};
use crate::knowledge::embedding::batching::{BatchConfig, BatchProcessor};
use crate::knowledge::embedding::metrics::EmbeddingMetrics;
use crate::knowledge::embedding::vector::DenseVector;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

/// Production embedding service configuration.
#[derive(Debug, Clone)]
pub struct EmbeddingServiceConfig {
    pub backend: BackendConfig,
    pub batch_size: usize,
    pub cache_enabled: bool,
    pub normalize: bool,
}

impl EmbeddingServiceConfig {
    /// Create with default configuration.
    pub fn new() -> Self {
        Self {
            backend: BackendConfig::default(),
            batch_size: 32,
            cache_enabled: true,
            normalize: true,
        }
    }

    /// Create with specific backend.
    pub fn with_backend(backend: impl Into<String>) -> Self {
        Self {
            backend: BackendConfig::new(backend),
            ..Default::default()
        }
    }
}

impl Default for EmbeddingServiceConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Production-grade embedding service.
pub struct EmbeddingService {
    backend: Arc<dyn EmbeddingBackend>,
    batch_processor: BatchProcessor,
    metrics: Arc<EmbeddingMetrics>,
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    config: EmbeddingServiceConfig,
}

impl EmbeddingService {
    /// Create a new embedding service.
    pub async fn new(config: EmbeddingServiceConfig) -> Result<Self, String> {
        // Create backend
        let backend = BackendRegistry::create(&config.backend)
            .map_err(|e| format!("Failed to create backend: {}", e))?;

        // Load backend
        backend
            .load()
            .await
            .map_err(|e| format!("Failed to load backend: {}", e))?;

        // Create batch processor
        let batch_config = BatchConfig::new(config.batch_size);
        let batch_processor = BatchProcessor::new(batch_config);

        Ok(Self {
            backend,
            batch_processor,
            metrics: Arc::new(EmbeddingMetrics::new()),
            cache: Arc::new(Mutex::new(HashMap::new())),
            config,
        })
    }

    /// Create with default configuration.
    pub async fn default() -> Result<Self, String> {
        Self::new(EmbeddingServiceConfig::default()).await
    }

    /// Embed a single text.
    pub async fn embed(&self, text: &str) -> Result<DenseVector, String> {
        let start = Instant::now();

        if text.is_empty() {
            return Err("Text cannot be empty".to_string());
        }

        // Check cache
        if self.config.cache_enabled {
            let cache = self.cache.lock().map_err(|e| e.to_string())?;
            if let Some(cached) = cache.get(text) {
                self.metrics.record_cache_hit();
                return Ok(cached.clone());
            }
        }

        self.metrics.record_cache_miss();

        // Generate embedding
        let mut embedding = self
            .backend
            .embed_text(text)
            .await
            .map_err(|e| format!("Backend embedding failed: {}", e))?;

        // Normalize if needed
        if self.config.normalize {
            self.backend.normalize(&mut embedding);
        }

        // Cache if enabled
        if self.config.cache_enabled {
            let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
            cache.insert(text.to_string(), embedding.clone());
        }

        // Record metrics
        let duration_ms = start.elapsed().as_millis() as u64;
        self.metrics.record_embedding(duration_ms);

        Ok(embedding)
    }

    /// Embed multiple texts (with batching).
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<DenseVector>, String> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let start = Instant::now();
        let batches = self.batch_processor.split_into_batches(texts);
        let mut results = Vec::with_capacity(texts.len());

        for batch in batches {
            let mut batch_embeddings = self
                .backend
                .embed_batch(&batch)
                .await
                .map_err(|e| format!("Batch embedding failed: {}", e))?;

            // Normalize if needed
            if self.config.normalize {
                for emb in batch_embeddings.iter_mut() {
                    self.backend.normalize(emb);
                }
            }

            // Cache if enabled
            if self.config.cache_enabled {
                let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
                for (text, embedding) in batch.iter().zip(batch_embeddings.iter()) {
                    cache.insert(text.to_string(), embedding.clone());
                }
            }

            results.extend(batch_embeddings);
        }

        // Record metrics
        let duration_ms = start.elapsed().as_millis() as u64;
        self.metrics
            .record_batch_embedding(texts.len() as u64, duration_ms);

        Ok(results)
    }

    /// Get backend information.
    pub fn backend_info(&self) -> BackendInfo {
        let meta = self.backend.metadata();
        BackendInfo {
            model_name: meta.model_name,
            dimension: meta.dimension,
            supports_batch: meta.supports_batch,
            supports_gpu: meta.supports_gpu,
            supports_cpu: meta.supports_cpu,
        }
    }

    /// Get metrics snapshot.
    pub fn metrics(&self) -> crate::knowledge::embedding::metrics::MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Clear cache.
    pub fn clear_cache(&self) -> Result<(), String> {
        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        cache.clear();
        Ok(())
    }

    /// Get cache size.
    pub fn cache_size(&self) -> Result<usize, String> {
        let cache = self.cache.lock().map_err(|e| e.to_string())?;
        Ok(cache.len())
    }
}

/// Backend information.
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub model_name: String,
    pub dimension: usize,
    pub supports_batch: bool,
    pub supports_gpu: bool,
    pub supports_cpu: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_service() {
        let config = EmbeddingServiceConfig::with_backend("local-hash");
        let service = EmbeddingService::new(config).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_embed_text() {
        let config = EmbeddingServiceConfig::with_backend("local-hash");
        let service = EmbeddingService::new(config).await.unwrap();
        let embedding = service.embed("hello world").await;
        assert!(embedding.is_ok());
        assert_eq!(embedding.unwrap().dimension(), 256);
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let config = EmbeddingServiceConfig::with_backend("local-hash");
        let service = EmbeddingService::new(config).await.unwrap();
        let texts = vec!["hello", "world", "test"];
        let embeddings = service.embed_batch(&texts).await;
        assert!(embeddings.is_ok());
        assert_eq!(embeddings.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_cache() {
        let config = EmbeddingServiceConfig::with_backend("local-hash");
        let service = EmbeddingService::new(config).await.unwrap();
        let _first = service.embed("cached").await.unwrap();
        let _second = service.embed("cached").await.unwrap();
        let cache_size = service.cache_size().unwrap();
        assert!(cache_size > 0);
    }

    #[tokio::test]
    async fn test_metrics() {
        let config = EmbeddingServiceConfig::with_backend("local-hash");
        let service = EmbeddingService::new(config).await.unwrap();
        let _embedding = service.embed("test").await.unwrap();
        let metrics = service.metrics();
        assert!(metrics.total_embeddings > 0);
    }
}
