//! Vector metadata storage and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata associated with a vector in the index.
///
/// This contains all information needed to reconstruct the original document
/// chunk and understand its context and provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    /// Unique identifier for the vector.
    pub id: String,
    /// Document ID this vector came from.
    pub document_id: String,
    /// Chunk ID within the document.
    pub chunk_id: usize,
    /// The actual text content of this chunk.
    pub content: String,
    /// Language of the content.
    pub language: String,
    /// Source of the document (e.g., file path, URL).
    pub source: String,
    /// Module or category this content belongs to.
    pub module: String,
    /// File path or location.
    pub path: String,
    /// Tags for classification.
    pub tags: Vec<String>,
    /// BLAKE3 hash of the content for deduplication.
    pub content_hash: String,
    /// Timestamp when this was indexed (Unix seconds).
    pub timestamp: u64,
    /// Version of the embedding model used.
    pub embedding_version: String,
    /// Author or creator of the document.
    pub author: String,
    /// Title of the document or section.
    pub title: String,
    /// Arbitrary additional metadata.
    pub extra: HashMap<String, String>,
}

impl VectorMetadata {
    /// Create a new metadata record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        document_id: String,
        chunk_id: usize,
        content: String,
        language: String,
        source: String,
        module: String,
        path: String,
        tags: Vec<String>,
        content_hash: String,
        embedding_version: String,
        author: String,
        title: String,
    ) -> Self {
        Self {
            id,
            document_id,
            chunk_id,
            content,
            language,
            source,
            module,
            path,
            tags,
            content_hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            embedding_version,
            author,
            title,
            extra: HashMap::new(),
        }
    }

    /// Create a minimal metadata record (useful for testing).
    pub fn minimal(id: String, document_id: String, content: String) -> Self {
        Self::new(
            id,
            document_id,
            0,
            content,
            "en".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
            vec![],
            "".to_string(),
            "1.0".to_string(),
            "unknown".to_string(),
            "".to_string(),
        )
    }

    /// Add an extra metadata field.
    pub fn with_extra(mut self, key: String, value: String) -> Self {
        self.extra.insert(key, value);
        self
    }

    /// Add multiple tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Get extra metadata value by key.
    pub fn get_extra(&self, key: &str) -> Option<&str> {
        self.extra.get(key).map(|s| s.as_str())
    }

    /// Check if this metadata matches a filter predicate.
    pub fn matches_filter<F>(&self, predicate: F) -> bool
    where
        F: Fn(&VectorMetadata) -> bool,
    {
        predicate(self)
    }
}

/// A collection of vector metadata indexed by vector ID.
pub struct MetadataStore {
    store: HashMap<String, VectorMetadata>,
}

impl MetadataStore {
    /// Create a new metadata store.
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Insert metadata for a vector.
    pub fn insert(&mut self, metadata: VectorMetadata) {
        self.store.insert(metadata.id.clone(), metadata);
    }

    /// Get metadata by vector ID.
    pub fn get(&self, id: &str) -> Option<&VectorMetadata> {
        self.store.get(id)
    }

    /// Get mutable metadata by vector ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut VectorMetadata> {
        self.store.get_mut(id)
    }

    /// Remove metadata by vector ID.
    pub fn remove(&mut self, id: &str) -> Option<VectorMetadata> {
        self.store.remove(id)
    }

    /// Get all metadata records.
    pub fn all(&self) -> Vec<&VectorMetadata> {
        self.store.values().collect()
    }

    /// Get number of stored metadata records.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Filter metadata by document ID.
    pub fn by_document(&self, document_id: &str) -> Vec<&VectorMetadata> {
        self.store
            .values()
            .filter(|m| m.document_id == document_id)
            .collect()
    }

    /// Filter metadata by tag.
    pub fn by_tag(&self, tag: &str) -> Vec<&VectorMetadata> {
        self.store
            .values()
            .filter(|m| m.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Filter metadata by module.
    pub fn by_module(&self, module: &str) -> Vec<&VectorMetadata> {
        self.store
            .values()
            .filter(|m| m.module == module)
            .collect()
    }

    /// Find metadata by predicate.
    pub fn find<F>(&self, predicate: F) -> Vec<&VectorMetadata>
    where
        F: Fn(&VectorMetadata) -> bool,
    {
        self.store
            .values()
            .filter(|m| predicate(m))
            .collect()
    }

    /// Clear all metadata.
    pub fn clear(&mut self) {
        self.store.clear();
    }
}

impl Default for MetadataStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let meta = VectorMetadata::minimal(
            "v1".to_string(),
            "d1".to_string(),
            "content".to_string(),
        );
        assert_eq!(meta.id, "v1");
        assert_eq!(meta.document_id, "d1");
    }

    #[test]
    fn test_metadata_store() {
        let mut store = MetadataStore::new();
        let meta = VectorMetadata::minimal(
            "v1".to_string(),
            "d1".to_string(),
            "content".to_string(),
        );
        store.insert(meta);
        assert_eq!(store.len(), 1);
        assert!(store.get("v1").is_some());
    }

    #[test]
    fn test_metadata_filtering() {
        let mut store = MetadataStore::new();
        for i in 0..3 {
            let mut meta = VectorMetadata::minimal(
                format!("v{}", i),
                format!("d{}", i % 2),
                format!("content {}", i),
            );
            meta.tags.push("test".to_string());
            store.insert(meta);
        }

        let by_doc = store.by_document("d0");
        assert!(by_doc.len() >= 1);

        let by_tag = store.by_tag("test");
        assert_eq!(by_tag.len(), 3);
    }
}
