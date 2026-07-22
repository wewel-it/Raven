//! Embedding cache with BLAKE3 content hashing.
//!
//! This module provides a cache that prevents re-embedding of identical content
//! by using BLAKE3 content hashing as the cache key.

use crate::knowledge::embedding::vector::DenseVector;
use blake3::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// A cached embedding entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEmbedding {
    /// BLAKE3 hash of the input content (stored as hex string for serialization).
    pub content_hash_hex: String,
    /// The resulting embedding vector.
    pub vector: DenseVector,
    /// When this entry was cached (Unix seconds).
    pub timestamp: u64,
}

impl CachedEmbedding {
    /// Create a new cached embedding entry.
    pub fn new(content_hash: Hash, vector: DenseVector) -> Self {
        Self {
            content_hash_hex: content_hash.to_hex().to_string(),
            vector,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

/// An embedding cache that uses BLAKE3 content hashing with interior mutability.
pub struct EmbeddingCache {
    cache: Arc<Mutex<HashMap<Hash, CachedEmbedding>>>,
    enabled: Arc<AtomicBool>,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
}

impl EmbeddingCache {
    /// Create a new embedding cache.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            enabled: Arc::new(AtomicBool::new(true)),
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a disabled cache (pass-through).
    pub fn disabled() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            enabled: Arc::new(AtomicBool::new(false)),
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Enable or disable caching.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if caching is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Compute BLAKE3 hash of content.
    pub fn hash_content(content: &str) -> Hash {
        blake3::hash(content.as_bytes())
    }

    /// Get a cached embedding by content hash.
    pub fn get(&self, content_hash: Hash) -> Option<DenseVector> {
        if !self.is_enabled() {
            return None;
        }

        let cache = self.cache.lock().ok()?;
        if let Some(cached) = cache.get(&content_hash) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(cached.vector.clone())
        } else {
            drop(cache);
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert an embedding into the cache.
    pub fn insert(&self, content_hash: Hash, vector: DenseVector) {
        if !self.is_enabled() {
            return;
        }
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(content_hash, CachedEmbedding::new(content_hash, vector));
        }
    }

    /// Get an embedding, or insert it if not cached.
    pub fn get_or_insert<F>(&self, content: &str, compute_fn: F) -> DenseVector
    where
        F: FnOnce() -> DenseVector,
    {
        if !self.is_enabled() {
            return compute_fn();
        }

        let hash = Self::hash_content(content);
        if let Some(embedding) = self.get(hash) {
            embedding
        } else {
            let embedding = compute_fn();
            self.insert(hash, embedding.clone());
            embedding
        }
    }

    /// Remove an entry from the cache.
    pub fn remove(&self, content_hash: Hash) -> Option<CachedEmbedding> {
        self.cache.lock().ok()?.remove(&content_hash)
    }

    /// Clear the entire cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// Get cache statistics.
    pub fn statistics(&self) -> EmbeddingCacheStats {
        let hit = self.hit_count.load(Ordering::Relaxed);
        let miss = self.miss_count.load(Ordering::Relaxed);
        let total = hit + miss;
        let hit_rate = if total == 0 {
            0.0
        } else {
            hit as f64 / total as f64
        };

        let len = self.cache.lock().ok().map(|c| c.len()).unwrap_or(0);

        EmbeddingCacheStats {
            total_entries: len,
            hit_count: hit,
            miss_count: miss,
            hit_rate,
        }
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.lock().ok().map(|c| c.len()).unwrap_or(0)
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.lock().ok().map(|c| c.is_empty()).unwrap_or(true)
    }

    /// Persist cache to file.
    pub fn persist(&self, path: &str) -> Result<(), String> {
        let cache = self.cache.lock().map_err(|e| e.to_string())?;
        let entries: Vec<_> = cache
            .iter()
            .map(|(hash, cached)| (hash.to_hex().to_string(), cached))
            .collect();
        let data = serde_json::to_string(&entries)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, data)
            .map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }

    /// Load cache from file.
    pub fn load(&self, path: &str) -> Result<usize, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("IO error: {}", e))?;
        let entries: Vec<(String, CachedEmbedding)> = serde_json::from_str(&data)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        for (hex_hash, cached) in entries {
            if let Ok(hash) = blake3::Hash::from_hex(&hex_hash) {
                cache.insert(hash, cached);
            }
        }
        Ok(cache.len())
    }
}

impl Default for EmbeddingCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about cache performance.
#[derive(Debug, Clone)]
pub struct EmbeddingCacheStats {
    pub total_entries: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        let mut cache = EmbeddingCache::new();
        let content = "hello world";
        let hash = EmbeddingCache::hash_content(content);
        let vector = DenseVector::new(vec![1.0, 2.0, 3.0]);

        cache.insert(hash, vector.clone());
        let result = cache.get(hash);
        assert!(result.is_some());
        assert_eq!(result.unwrap().data(), vector.data());
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = EmbeddingCache::new();
        let hash = EmbeddingCache::hash_content("test");
        assert!(cache.get(hash).is_none());
    }

    #[test]
    fn test_cache_disabled() {
        let mut cache = EmbeddingCache::disabled();
        let hash = EmbeddingCache::hash_content("test");
        let vector = DenseVector::new(vec![1.0, 2.0]);
        cache.insert(hash, vector);
        assert!(cache.get(hash).is_none());
    }

    #[test]
    fn test_get_or_insert() {
        let mut cache = EmbeddingCache::new();
        let content = "test content";
        let vector = DenseVector::new(vec![1.0, 2.0, 3.0]);

        let result = cache.get_or_insert(content, || vector.clone());
        assert_eq!(result.data(), vector.data());

        // Second call should hit cache
        let result2 = cache.get_or_insert(content, || panic!("Should not be called"));
        assert_eq!(result2.data(), vector.data());
    }

    #[test]
    fn test_cache_statistics() {
        let mut cache = EmbeddingCache::new();
        let hash = EmbeddingCache::hash_content("test");
        let vector = DenseVector::new(vec![1.0]);

        cache.insert(hash, vector);
        cache.get(hash);
        cache.get(blake3::hash(b"other"));

        let stats = cache.statistics();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
    }
}
