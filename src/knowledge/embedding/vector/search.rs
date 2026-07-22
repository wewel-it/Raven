//! Search result types and operations.

use crate::knowledge::embedding::vector::storage::StoredVector;
use serde::{Deserialize, Serialize};

/// A single search result entry with similarity score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub vector_id: String,
    pub similarity_score: f32,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl SearchResult {
    /// Create a new search result from a stored vector.
    pub fn from_stored(vector_id: String, similarity_score: f32, stored: &StoredVector) -> Self {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("document_id".to_string(), stored.metadata.document_id.clone());
        metadata.insert("chunk_id".to_string(), stored.metadata.chunk_id.to_string());
        metadata.insert("language".to_string(), stored.metadata.language.clone());
        metadata.insert("source".to_string(), stored.metadata.source.clone());
        metadata.insert("module".to_string(), stored.metadata.module.clone());
        metadata.insert("tags".to_string(), stored.metadata.tags.join(","));

        Self {
            vector_id,
            similarity_score,
            content: stored.metadata.content.clone(),
            metadata,
        }
    }
}

/// Collection of search results with ranking and statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultSet {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub total_searched: usize,
    pub total_candidates: usize,
}

impl SearchResultSet {
    /// Create a new search result set.
    pub fn new(
        results: Vec<SearchResult>,
        query: String,
        total_searched: usize,
        total_candidates: usize,
    ) -> Self {
        Self {
            results,
            query,
            total_searched,
            total_candidates,
        }
    }

    /// Get the number of results.
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Check if result set is empty.
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the top-k results.
    pub fn top_k(&self, k: usize) -> Vec<&SearchResult> {
        self.results.iter().take(k).collect()
    }

    /// Get the best (first) result.
    pub fn best(&self) -> Option<&SearchResult> {
        self.results.first()
    }

    /// Get average similarity score.
    pub fn average_score(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            self.results.iter().map(|r| r.similarity_score).sum::<f32>() / self.results.len() as f32
        }
    }

    /// Get minimum similarity score.
    pub fn min_score(&self) -> f32 {
        self.results
            .iter()
            .map(|r| r.similarity_score)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get maximum similarity score.
    pub fn max_score(&self) -> f32 {
        self.results
            .iter()
            .map(|r| r.similarity_score)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Filter results by minimum similarity score.
    pub fn filter_by_score(&self, min_score: f32) -> Vec<SearchResult> {
        self.results
            .iter()
            .filter(|r| r.similarity_score >= min_score)
            .cloned()
            .collect()
    }

    /// Filter results by module.
    pub fn filter_by_module(&self, module: &str) -> Vec<SearchResult> {
        self.results
            .iter()
            .filter(|r| {
                r.metadata
                    .get("module")
                    .map(|m| m == module)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Deduplicate results by document ID (keep highest score).
    pub fn deduplicate_by_document(&self) -> SearchResultSet {
        let mut best_per_doc: std::collections::HashMap<String, SearchResult> =
            std::collections::HashMap::new();
        for result in &self.results {
            if let Some(doc_id) = result.metadata.get("document_id") {
                best_per_doc
                    .entry(doc_id.clone())
                    .and_modify(|e| {
                        if result.similarity_score > e.similarity_score {
                            *e = result.clone();
                        }
                    })
                    .or_insert_with(|| result.clone());
            }
        }

        let mut deduped: Vec<_> = best_per_doc.into_values().collect();
        deduped.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        SearchResultSet::new(
            deduped,
            self.query.clone(),
            self.total_searched,
            self.total_candidates,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_set() {
        let results = vec![
            SearchResult {
                vector_id: "v1".to_string(),
                similarity_score: 0.9,
                content: "content1".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            SearchResult {
                vector_id: "v2".to_string(),
                similarity_score: 0.7,
                content: "content2".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        ];
        let set = SearchResultSet::new(results, "query".to_string(), 2, 100);
        assert_eq!(set.len(), 2);
        assert!((set.average_score() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_top_k() {
        let results = vec![
            SearchResult {
                vector_id: "v1".to_string(),
                similarity_score: 0.9,
                content: "content1".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            SearchResult {
                vector_id: "v2".to_string(),
                similarity_score: 0.7,
                content: "content2".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            SearchResult {
                vector_id: "v3".to_string(),
                similarity_score: 0.5,
                content: "content3".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        ];
        let set = SearchResultSet::new(results, "query".to_string(), 3, 100);
        let top2 = set.top_k(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].vector_id, "v1");
    }
}
