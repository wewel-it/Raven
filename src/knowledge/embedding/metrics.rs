//! Embedding metrics and telemetry.
//!
//! This module provides metrics collection for embedding operations.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Embedding metrics.
#[derive(Debug, Clone)]
pub struct EmbeddingMetrics {
    total_embeddings: Arc<AtomicU64>,
    total_batch_embeddings: Arc<AtomicU64>,
    total_embedding_time_ms: Arc<AtomicU64>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    model_load_count: Arc<AtomicU64>,
    total_model_load_time_ms: Arc<AtomicU64>,
}

impl EmbeddingMetrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self {
            total_embeddings: Arc::new(AtomicU64::new(0)),
            total_batch_embeddings: Arc::new(AtomicU64::new(0)),
            total_embedding_time_ms: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            model_load_count: Arc::new(AtomicU64::new(0)),
            total_model_load_time_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record single embedding.
    pub fn record_embedding(&self, duration_ms: u64) {
        self.total_embeddings.fetch_add(1, Ordering::SeqCst);
        self.total_embedding_time_ms
            .fetch_add(duration_ms, Ordering::SeqCst);
    }

    /// Record batch embedding.
    pub fn record_batch_embedding(&self, count: u64, duration_ms: u64) {
        self.total_batch_embeddings
            .fetch_add(count, Ordering::SeqCst);
        self.total_embedding_time_ms
            .fetch_add(duration_ms, Ordering::SeqCst);
    }

    /// Record cache hit.
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::SeqCst);
    }

    /// Record cache miss.
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::SeqCst);
    }

    /// Record model load.
    pub fn record_model_load(&self, duration_ms: u64) {
        self.model_load_count.fetch_add(1, Ordering::SeqCst);
        self.total_model_load_time_ms
            .fetch_add(duration_ms, Ordering::SeqCst);
    }

    /// Get total embeddings.
    pub fn total_embeddings(&self) -> u64 {
        self.total_embeddings.load(Ordering::SeqCst)
    }

    /// Get total batch embeddings.
    pub fn total_batch_embeddings(&self) -> u64 {
        self.total_batch_embeddings.load(Ordering::SeqCst)
    }

    /// Get average embedding time.
    pub fn average_embedding_time_ms(&self) -> f64 {
        let total_time = self.total_embedding_time_ms.load(Ordering::SeqCst) as f64;
        let total_count = self.total_embeddings() as f64;
        if total_count > 0.0 {
            total_time / total_count
        } else {
            0.0
        }
    }

    /// Get cache hit ratio.
    pub fn cache_hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::SeqCst) as f64;
        let misses = self.cache_misses.load(Ordering::SeqCst) as f64;
        let total = hits + misses;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }

    /// Get average model load time.
    pub fn average_model_load_time_ms(&self) -> f64 {
        let total_time = self.total_model_load_time_ms.load(Ordering::SeqCst) as f64;
        let count = self.model_load_count.load(Ordering::SeqCst) as f64;
        if count > 0.0 {
            total_time / count
        } else {
            0.0
        }
    }

    /// Get metrics snapshot.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_embeddings: self.total_embeddings(),
            total_batch_embeddings: self.total_batch_embeddings(),
            average_embedding_time_ms: self.average_embedding_time_ms(),
            cache_hits: self.cache_hits.load(Ordering::SeqCst),
            cache_misses: self.cache_misses.load(Ordering::SeqCst),
            cache_hit_ratio: self.cache_hit_ratio(),
            model_load_count: self.model_load_count.load(Ordering::SeqCst),
            average_model_load_time_ms: self.average_model_load_time_ms(),
        }
    }

    /// Reset metrics.
    pub fn reset(&self) {
        self.total_embeddings.store(0, Ordering::SeqCst);
        self.total_batch_embeddings.store(0, Ordering::SeqCst);
        self.total_embedding_time_ms.store(0, Ordering::SeqCst);
        self.cache_hits.store(0, Ordering::SeqCst);
        self.cache_misses.store(0, Ordering::SeqCst);
        self.model_load_count.store(0, Ordering::SeqCst);
        self.total_model_load_time_ms.store(0, Ordering::SeqCst);
    }
}

impl Default for EmbeddingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics snapshot at a point in time.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_embeddings: u64,
    pub total_batch_embeddings: u64,
    pub average_embedding_time_ms: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_ratio: f64,
    pub model_load_count: u64,
    pub average_model_load_time_ms: f64,
}

impl MetricsSnapshot {
    /// Convert snapshot to string representation.
    pub fn to_string(&self) -> String {
        format!(
            "EmbeddingMetrics {{ total_embeddings: {}, total_batch: {}, avg_time: {:.2}ms, \
             cache_hit_ratio: {:.2}%, model_loads: {} }}",
            self.total_embeddings,
            self.total_batch_embeddings,
            self.average_embedding_time_ms,
            self.cache_hit_ratio * 100.0,
            self.model_load_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = EmbeddingMetrics::new();
        assert_eq!(metrics.total_embeddings(), 0);
    }

    #[test]
    fn test_record_embedding() {
        let metrics = EmbeddingMetrics::new();
        metrics.record_embedding(10);
        assert_eq!(metrics.total_embeddings(), 1);
    }

    #[test]
    fn test_cache_hit_ratio() {
        let metrics = EmbeddingMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        let ratio = metrics.cache_hit_ratio();
        assert!((ratio - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_average_time() {
        let metrics = EmbeddingMetrics::new();
        metrics.record_embedding(10);
        metrics.record_embedding(20);
        let avg = metrics.average_embedding_time_ms();
        assert!((avg - 15.0).abs() < 0.001);
    }
}
