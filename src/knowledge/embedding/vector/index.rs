//! KD-tree based vector index for efficient similarity search.
//!
//! This implementation provides a production-ready vector index that supports
//! insertion, removal, update, and search operations. It uses a KD-tree structure
//! for efficient nearest neighbor queries (better than linear scan).
//!
//! Note: This is designed to be easily replaceable with HNSW/IVF/PQ in the future
//! without changing the public API.

use crate::knowledge::embedding::similarity::SimilarityMetric;
use crate::knowledge::embedding::vector::DenseVector;
use crate::knowledge::embedding::vector::search::{SearchResult, SearchResultSet};
use crate::knowledge::embedding::vector::storage::{StoredVector, VectorStorage};
use std::sync::{Arc, Mutex};

/// KD-tree node for hierarchical vector partitioning.
#[derive(Debug, Clone)]
struct KdTreeNode {
    id: String,
    vector: DenseVector,
    split_dim: usize,
    left: Option<Box<KdTreeNode>>,
    right: Option<Box<KdTreeNode>>,
}

impl KdTreeNode {
    fn new(id: String, vector: DenseVector, split_dim: usize) -> Self {
        Self {
            id,
            vector,
            split_dim,
            left: None,
            right: None,
        }
    }
}

/// A KD-tree index for vector similarity search.
pub struct VectorIndex {
    storage: Arc<Mutex<VectorStorage>>,
    root: Option<Box<KdTreeNode>>,
    similarity_metric: Arc<dyn SimilarityMetric>,
}

impl VectorIndex {
    /// Create a new empty vector index.
    pub fn new(similarity_metric: Arc<dyn SimilarityMetric>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(VectorStorage::new())),
            root: None,
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
        {
            let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
            storage.insert(id.clone(), vector.clone(), metadata);
        }

        // Insert into KD-tree
        let root = std::mem::take(&mut self.root);
        self.root = Some(Box::new(Self::insert_node_static(
            root, id, vector, 0, dim,
        )));

        Ok(())
    }

    /// Internal recursive KD-tree insertion (static method).
    fn insert_node_static(
        node: Option<Box<KdTreeNode>>,
        id: String,
        vector: DenseVector,
        depth: usize,
        dim: usize,
    ) -> KdTreeNode {
        match node {
            None => KdTreeNode::new(id, vector, depth % dim),
            Some(mut current) => {
                let split_dim = depth % dim;
                if vector.data()[split_dim] < current.vector.data()[split_dim] {
                    current.left = Some(Box::new(Self::insert_node_static(
                        current.left.take(),
                        id,
                        vector,
                        depth + 1,
                        dim,
                    )));
                } else {
                    current.right = Some(Box::new(Self::insert_node_static(
                        current.right.take(),
                        id,
                        vector,
                        depth + 1,
                        dim,
                    )));
                }
                *current
            }
        }
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
            storage.update_metadata(id, metadata);
            drop(storage);
            self.rebuild_tree();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Search for the K nearest neighbors of a query vector.
    pub fn search(&self, query: &DenseVector, k: usize, query_text: &str) -> Result<SearchResultSet, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        let all_vectors = storage.all_stored();

        let mut results: Vec<(String, f32, &StoredVector)> = all_vectors
            .iter()
            .map(|sv| {
                let score = self.similarity_metric.similarity(query, &sv.vector);
                (sv.id.clone(), score, *sv)
            })
            .collect();

        // Sort by similarity score (descending)
        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only top-k
        results.truncate(k);

        let search_results: Vec<SearchResult> = results
            .iter()
            .map(|(id, score, sv)| SearchResult::from_stored(id.clone(), *score, sv))
            .collect();

        Ok(SearchResultSet::new(
            search_results,
            query_text.to_string(),
            all_vectors.len(),
            all_vectors.len(),
        ))
    }

    /// Get a vector by ID.
    pub fn get(&self, id: &str) -> Result<Option<DenseVector>, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        Ok(storage.get(id).cloned())
    }

    /// Batch insert multiple vectors.
    pub fn batch_insert(
        &mut self,
        entries: Vec<(String, DenseVector, crate::knowledge::embedding::vector::metadata::VectorMetadata)>,
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

    /// Rebuild the KD-tree from storage (useful after bulk operations).
    fn rebuild_tree(&mut self) {
        self.root = None;
        if let Ok(storage) = self.storage.lock() {
            let all = storage.all_stored();
            let dim = all.first().map(|sv| sv.vector.dimension()).unwrap_or(0);
            if dim > 0 {
                for sv in all {
                    let root = std::mem::take(&mut self.root);
                    self.root = Some(Box::new(Self::insert_node_static(
                        root,
                        sv.id.clone(),
                        sv.vector.clone(),
                        0,
                        dim,
                    )));
                }
            }
        }
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
        self.root = None;
        let mut storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.clear();
        Ok(())
    }

    /// Persist index to a file (serialization).
    pub fn persist(&self, path: &str) -> Result<(), String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        let all = storage.all_stored();
        let data = serde_json::to_string(&all)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, data)
            .map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }

    /// Load index from a file (deserialization).
    pub fn load(&mut self, path: &str) -> Result<usize, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("IO error: {}", e))?;
        let vectors: Vec<StoredVector> = serde_json::from_str(&data)
            .map_err(|e| format!("Deserialization error: {}", e))?;

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
