//! In-memory vector storage with metadata.

use crate::knowledge::embedding::vector::DenseVector;
use crate::knowledge::embedding::vector::metadata::{MetadataStore, VectorMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single stored vector entry with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredVector {
    pub id: String,
    pub vector: DenseVector,
    pub metadata: VectorMetadata,
}

/// In-memory vector storage that supports insertion, retrieval, and bulk operations.
pub struct VectorStorage {
    vectors: HashMap<String, StoredVector>,
    metadata_store: MetadataStore,
}

impl VectorStorage {
    /// Create a new empty vector storage.
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            metadata_store: MetadataStore::new(),
        }
    }

    /// Insert a vector with metadata.
    pub fn insert(&mut self, id: String, vector: DenseVector, metadata: VectorMetadata) {
        let stored = StoredVector {
            id: id.clone(),
            vector,
            metadata: metadata.clone(),
        };
        self.vectors.insert(id.clone(), stored);
        self.metadata_store.insert(metadata);
    }

    /// Get a vector by ID.
    pub fn get(&self, id: &str) -> Option<&DenseVector> {
        self.vectors.get(id).map(|sv| &sv.vector)
    }

    /// Get a stored vector (vector + metadata) by ID.
    pub fn get_stored(&self, id: &str) -> Option<&StoredVector> {
        self.vectors.get(id)
    }

    /// Remove a vector by ID.
    pub fn remove(&mut self, id: &str) -> Option<StoredVector> {
        self.metadata_store.remove(id);
        self.vectors.remove(id)
    }

    /// Check if a vector exists.
    pub fn contains(&self, id: &str) -> bool {
        self.vectors.contains_key(id)
    }

    /// Get the total number of stored vectors.
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if the storage is empty.
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Get all vector IDs.
    pub fn ids(&self) -> Vec<String> {
        self.vectors.keys().cloned().collect()
    }

    /// Get all stored vectors.
    pub fn all_stored(&self) -> Vec<&StoredVector> {
        self.vectors.values().collect()
    }

    /// Batch insert multiple vectors.
    pub fn batch_insert(
        &mut self,
        entries: Vec<(String, DenseVector, VectorMetadata)>,
    ) {
        for (id, vector, metadata) in entries {
            self.insert(id, vector, metadata);
        }
    }

    /// Get vectors by document ID.
    pub fn by_document(&self, document_id: &str) -> Vec<&StoredVector> {
        self.vectors
            .values()
            .filter(|sv| sv.metadata.document_id == document_id)
            .collect()
    }

    /// Get vectors by tag.
    pub fn by_tag(&self, tag: &str) -> Vec<&StoredVector> {
        self.vectors
            .values()
            .filter(|sv| sv.metadata.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get vectors by module.
    pub fn by_module(&self, module: &str) -> Vec<&StoredVector> {
        self.vectors
            .values()
            .filter(|sv| sv.metadata.module == module)
            .collect()
    }

    /// Find vectors matching a predicate.
    pub fn find<F>(&self, predicate: F) -> Vec<&StoredVector>
    where
        F: Fn(&StoredVector) -> bool,
    {
        self.vectors.values().filter(|sv| predicate(sv)).collect()
    }

    /// Get metadata store reference.
    pub fn metadata_store(&self) -> &MetadataStore {
        &self.metadata_store
    }

    /// Get mutable metadata store reference.
    pub fn metadata_store_mut(&mut self) -> &mut MetadataStore {
        &mut self.metadata_store
    }

    /// Clear all vectors.
    pub fn clear(&mut self) {
        self.vectors.clear();
        self.metadata_store.clear();
    }

    /// Update metadata for a vector.
    pub fn update_metadata(&mut self, id: &str, metadata: VectorMetadata) -> bool {
        if let Some(stored) = self.vectors.get_mut(id) {
            stored.metadata = metadata.clone();
            self.metadata_store.insert(metadata);
            true
        } else {
            false
        }
    }

    /// Get statistics about stored vectors.
    pub fn statistics(&self) -> VectorStorageStatistics {
        let total = self.vectors.len();
        let dimension = if total > 0 {
            self.vectors
                .values()
                .next()
                .map(|sv| sv.vector.dimension())
                .unwrap_or(0)
        } else {
            0
        };

        let modules: std::collections::HashSet<_> = self
            .vectors
            .values()
            .map(|sv| sv.metadata.module.clone())
            .collect();

        let documents: std::collections::HashSet<_> = self
            .vectors
            .values()
            .map(|sv| sv.metadata.document_id.clone())
            .collect();

        VectorStorageStatistics {
            total_vectors: total,
            vector_dimension: dimension,
            unique_modules: modules.len(),
            unique_documents: documents.len(),
        }
    }
}

impl Default for VectorStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the vector storage.
#[derive(Debug, Clone)]
pub struct VectorStorageStatistics {
    pub total_vectors: usize,
    pub vector_dimension: usize,
    pub unique_modules: usize,
    pub unique_documents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_retrieve() {
        let mut storage = VectorStorage::new();
        let vector = DenseVector::new(vec![1.0, 2.0, 3.0]);
        let metadata = VectorMetadata::minimal(
            "v1".to_string(),
            "d1".to_string(),
            "content".to_string(),
        );

        storage.insert("v1".to_string(), vector.clone(), metadata);
        assert!(storage.contains("v1"));
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut storage = VectorStorage::new();
        let vector = DenseVector::new(vec![1.0, 2.0]);
        let metadata =
            VectorMetadata::minimal("v1".to_string(), "d1".to_string(), "x".to_string());
        storage.insert("v1".to_string(), vector, metadata);
        assert_eq!(storage.len(), 1);

        storage.remove("v1");
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_batch_operations() {
        let mut storage = VectorStorage::new();
        let mut entries = Vec::new();
        for i in 0..5 {
            let vector = DenseVector::new(vec![i as f32]);
            let metadata = VectorMetadata::minimal(
                format!("v{}", i),
                format!("d{}", i),
                format!("content {}", i),
            );
            entries.push((format!("v{}", i), vector, metadata));
        }
        storage.batch_insert(entries);
        assert_eq!(storage.len(), 5);
    }

    #[test]
    fn test_filtering() {
        let mut storage = VectorStorage::new();
        for i in 0..3 {
            let vector = DenseVector::new(vec![i as f32]);
            let mut metadata = VectorMetadata::minimal(
                format!("v{}", i),
                "d1".to_string(),
                format!("content {}", i),
            );
            metadata.tags.push("test".to_string());
            storage.insert(format!("v{}", i), vector, metadata);
        }

        let by_doc = storage.by_document("d1");
        assert_eq!(by_doc.len(), 3);

        let by_tag = storage.by_tag("test");
        assert_eq!(by_tag.len(), 3);
    }

    #[test]
    fn test_statistics() {
        let mut storage = VectorStorage::new();
        let vector = DenseVector::new(vec![1.0, 2.0, 3.0]);
        let metadata = VectorMetadata::minimal(
            "v1".to_string(),
            "d1".to_string(),
            "content".to_string(),
        );
        storage.insert("v1".to_string(), vector, metadata);

        let stats = storage.statistics();
        assert_eq!(stats.total_vectors, 1);
        assert_eq!(stats.vector_dimension, 3);
    }
}
