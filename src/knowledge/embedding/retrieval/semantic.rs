//! Semantic search using embeddings.

use crate::knowledge::embedding::engine::EmbeddingEngine;
use crate::knowledge::embedding::vector::VectorIndex;
use crate::knowledge::embedding::vector::search::SearchResultSet;
use std::sync::Arc;

/// A semantic search engine that uses embeddings and vector similarity.
pub struct SemanticSearchEngine {
    embedding_engine: Arc<dyn EmbeddingEngine>,
    vector_index: Arc<parking_lot::RwLock<VectorIndex>>,
}

impl SemanticSearchEngine {
    /// Create a new semantic search engine.
    pub fn new(
        embedding_engine: Arc<dyn EmbeddingEngine>,
        vector_index: Arc<parking_lot::RwLock<VectorIndex>>,
    ) -> Self {
        Self {
            embedding_engine,
            vector_index,
        }
    }

    /// Perform a semantic search query.
    pub fn search(&self, query: &str, k: usize) -> Result<SearchResultSet, String> {
        // Embed the query
        let query_embedding = self
            .embedding_engine
            .embed_query(query)
            .map_err(|e| format!("Failed to embed query: {}", e))?;

        // Search in the vector index
        let index = self.vector_index.read();
        index.search(&query_embedding, k, query)
    }

    /// Perform a semantic search with a minimum similarity threshold.
    pub fn search_with_threshold(
        &self,
        query: &str,
        k: usize,
        min_score: f32,
    ) -> Result<SearchResultSet, String> {
        let results = self.search(query, k)?;
        let filtered = results.filter_by_score(min_score);
        Ok(SearchResultSet::new(
            filtered,
            results.query,
            results.total_searched,
            results.total_candidates,
        ))
    }

    /// Perform a semantic search with module filtering.
    pub fn search_by_module(
        &self,
        query: &str,
        k: usize,
        module: &str,
    ) -> Result<SearchResultSet, String> {
        let results = self.search(query, k)?;
        let filtered = results.filter_by_module(module);
        Ok(SearchResultSet::new(
            filtered,
            results.query,
            results.total_searched,
            results.total_candidates,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::embedding::engine::LocalEmbeddingEngine;
    use crate::knowledge::embedding::similarity::CosineSimilarity;
    use crate::knowledge::embedding::vector::metadata::VectorMetadata;
    use std::sync::Arc;

    #[test]
    fn test_semantic_search() {
        let docs = vec!["hello world", "test document", "semantic search"];
        let engine = Arc::new(
            LocalEmbeddingEngine::with_documents(&docs)
                .expect("Failed to create engine"),
        );

        let metric = Arc::new(CosineSimilarity);
        let index = Arc::new(parking_lot::RwLock::new(VectorIndex::new(metric)));

        // Insert vectors
        {
            let mut idx = index.write();
            for (i, doc) in docs.iter().enumerate() {
                let embedding = engine.embed_text(doc).unwrap();
                let meta = VectorMetadata::minimal(
                    format!("v{}", i),
                    format!("d{}", i),
                    doc.to_string(),
                );
                idx.insert(format!("v{}", i), embedding, meta)
                    .expect("Failed to insert");
            }
        }

        let search_engine = SemanticSearchEngine::new(engine, index);
        let results = search_engine.search("hello world", 2).unwrap();
        assert!(results.len() > 0);
    }
}
