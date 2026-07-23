//! Vector index backed by in-memory storage with search capabilities.
//!
// This implementation provides a stable public API and currently performs
// exact nearest-neighbor search by scanning stored vectors. It's designed
// to be replaceable by a more advanced index (KD-tree, HNSW) in future.

use crate::knowledge::embedding::similarity::SimilarityMetric;
use crate::knowledge::embedding::vector::search::{SearchResult, SearchResultSet};
use crate::knowledge::embedding::vector::storage::{StoredVector, VectorStorage};
use crate::knowledge::embedding::vector::DenseVector;
use std::sync::{Arc, Mutex};

/// A production-ready vector index for similarity search.
pub struct VectorIndex {
    storage: Arc<Mutex<VectorStorage>>,
    similarity_metric: Arc<dyn SimilarityMetric>,
}

/// Options to control vector search behavior.
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub min_score: Option<f32>,
    pub metadata_filter: Option<std::collections::HashMap<String, String>>,
    pub namespace: Option<String>,
    pub language: Option<String>,
}

impl VectorIndex {
    /// Create a new empty vector index.
    pub fn new(similarity_metric: Arc<dyn SimilarityMetric>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(VectorStorage::new())),
            similarity_metric,
        }
    }

    /// Insert a vector with metadata into the index.
    pub fn insert(
        &mut self,
        id: String,
        vector: DenseVector,
        metadata: crate::knowledge::embedding::vector::metadata::VectorMetadata,
    ) -> Result<(), String> {
        let dim = vector.dimension();
        if dim == 0 {
            return Err("Cannot insert zero-dimensional vector".to_string());
        }

        // Insert into storage
        let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.insert(id, vector, metadata);
        Ok(())
    }

    /// Remove a vector from the index.
    pub fn remove(&mut self, id: &str) -> Result<bool, String> {
        {
            let storage = self.storage.lock().map_err(|e| e.to_string())?;
            if !storage.contains(id) {
                return Ok(false);
            }
        }

        {
            let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
            storage.remove(id);
        }

        self.rebuild_tree();
        Ok(true)
    }

    /// Update a vector in the index.
    pub fn update(
        &mut self,
        id: &str,
        vector: DenseVector,
        metadata: crate::knowledge::embedding::vector::metadata::VectorMetadata,
    ) -> Result<bool, String> {
        let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
        if storage.contains(id) {
            storage.insert(id.to_string(), vector, metadata);
            drop(storage);
            self.rebuild_tree();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Search for the K nearest neighbors of a query vector (default options).
    pub fn search(
        &self,
        query: &DenseVector,
        k: usize,
        query_text: &str,
    ) -> Result<SearchResultSet, String> {
        self.search_with_options(query, k, query_text, &SearchOptions::default())
    }

    /// Search with explicit options for filtering and scoring.
    pub fn search_with_options(
        &self,
        query: &DenseVector,
        k: usize,
        query_text: &str,
        opts: &SearchOptions,
    ) -> Result<SearchResultSet, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        let all_vectors = storage.all_stored();

        // Compute raw scores and apply metadata/language/namespace filters
        let mut scored: Vec<(String, f32, &StoredVector)> = Vec::new();
        for sv in &all_vectors {
            // Namespace/module filter
            if let Some(ns) = &opts.namespace {
                if sv.metadata.module != *ns {
                    continue;
                }
            }

            // Language filter
            if let Some(lang) = &opts.language {
                if sv.metadata.language != *lang {
                    continue;
                }
            }

            // Metadata key-value filters
            if let Some(map) = &opts.metadata_filter {
                let mut matched = true;
                for (k, v) in map.iter() {
                    if let Some(val) = sv.metadata.extra.get(k) {
                        if val != v {
                            matched = false;
                            break;
                        }
                    } else {
                        matched = false;
                        break;
                    }
                }
                if !matched {
                    continue;
                }
            }

            let score = self.similarity_metric.similarity(query, &sv.vector);
            scored.push((sv.id.clone(), score, sv));
        }

        // If no candidates, return empty set
        if scored.is_empty() {
            return Ok(SearchResultSet::new(vec![], query_text.to_string(), 0, 0));
        }

        // Normalize by top score
        let mut max_score = scored.iter().map(|(_, s, _)| *s).fold(f32::MIN, f32::max);
        if max_score.is_nan() || max_score <= 0.0 {
            // avoid division by zero; use absolute max
            max_score = scored.iter().map(|(_, s, _)| s.abs()).fold(0.0, f32::max);
            if max_score <= 0.0 {
                max_score = 1.0;
            }
        }

        let mut results: Vec<SearchResult> = scored
            .into_iter()
            .map(|(id, raw_score, sv)| {
                let mut norm = raw_score / max_score;
                if norm.is_nan() || norm.is_infinite() {
                    norm = 0.0;
                }
                norm = norm.clamp(0.0, 1.0);
                SearchResult::from_stored(id, norm, sv)
            })
            .collect();

        // Apply min_score filter if present
        if let Some(min) = opts.min_score {
            results.retain(|r| r.similarity_score >= min);
        }

        // Sort and take top-k
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let total_candidates = results.len();
        results.truncate(k);

        Ok(SearchResultSet::new(
            results,
            query_text.to_string(),
            total_candidates,
            total_candidates,
        ))
    }

    /// Batch insert multiple vectors.
    pub fn batch_insert(
        &mut self,
        entries: Vec<(
            String,
            DenseVector,
            crate::knowledge::embedding::vector::metadata::VectorMetadata,
        )>,
    ) -> Result<usize, String> {
        let dim = entries.first().map(|(_, v, _)| v.dimension()).unwrap_or(0);
        if dim == 0 && !entries.is_empty() {
            return Err("Cannot insert zero-dimensional vectors".to_string());
        }

        let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
        for (id, vector, metadata) in &entries {
            storage.insert(id.clone(), vector.clone(), metadata.clone());
        }
        drop(storage);

        self.rebuild_tree();
        Ok(entries.len())
    }

    /// Rebuild the index from storage.
    ///
    /// Currently, the vector storage is authoritative and search uses exact scanning.
    /// This method is kept for API compatibility and future index structures.
    fn rebuild_tree(&mut self) {
        // No-op for storage-backed exact search.
    }

    /// Get the total number of vectors in the index.
    pub fn len(&self) -> Result<usize, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        Ok(storage.len())
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> Result<bool, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        Ok(storage.is_empty())
    }

    /// Clear all vectors from the index.
    pub fn clear(&mut self) -> Result<(), String> {
        let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.clear();
        Ok(())
    }

    /// Persist index to a file (serialization).
    pub fn persist(&self, path: &str) -> Result<(), String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        let all = storage.all_stored();
        let data =
            serde_json::to_string(&all).map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, data).map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }

    /// Load index from a file (deserialization).
    pub fn load(&mut self, path: &str) -> Result<usize, String> {
        let data = std::fs::read_to_string(path).map_err(|e| format!("IO error: {}", e))?;
        let vectors: Vec<StoredVector> =
            serde_json::from_str(&data).map_err(|e| format!("Deserialization error: {}", e))?;

        let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.clear();
        for sv in &vectors {
            storage.insert(sv.id.clone(), sv.vector.clone(), sv.metadata.clone());
        }
        drop(storage);

        self.rebuild_tree();
        Ok(vectors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::embedding::similarity::CosineSimilarity;
    use crate::knowledge::embedding::vector::metadata::VectorMetadata;

    #[test]
    fn test_index_creation() {
        let metric = Arc::new(CosineSimilarity);
        let index = VectorIndex::new(metric);
        assert!(index.is_empty().unwrap());
    }

    #[test]
    fn test_insert_and_search() {
        let metric = Arc::new(CosineSimilarity);
        let mut index = VectorIndex::new(metric);

        let v1 = DenseVector::new(vec![1.0, 0.0]).normalize();
        let v2 = DenseVector::new(vec![0.0, 1.0]).normalize();

        let meta1 =
            VectorMetadata::minimal("id1".to_string(), "doc1".to_string(), "text1".to_string());
        let meta2 =
            VectorMetadata::minimal("id2".to_string(), "doc2".to_string(), "text2".to_string());

        assert!(index.insert("id1".to_string(), v1.clone(), meta1).is_ok());
        assert!(index.insert("id2".to_string(), v2.clone(), meta2).is_ok());
        assert_eq!(index.len().unwrap(), 2);

        let query = DenseVector::new(vec![1.0, 0.0]).normalize();
        let results = index.search(&query, 2, "test").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_remove() {
        let metric = Arc::new(CosineSimilarity);
        let mut index = VectorIndex::new(metric);

        let v = DenseVector::new(vec![1.0, 2.0]);
        let meta =
            VectorMetadata::minimal("id1".to_string(), "doc1".to_string(), "text".to_string());
        index.insert("id1".to_string(), v, meta).unwrap();
        assert_eq!(index.len().unwrap(), 1);

        index.remove("id1").unwrap();
        assert_eq!(index.len().unwrap(), 0);
    }
}
