//! Hybrid retrieval combining semantic and keyword search.

use crate::knowledge::embedding::vector::search::{SearchResult, SearchResultSet};

/// Hybrid retrieval configuration.
#[derive(Debug, Clone)]
pub struct HybridRetrievalConfig {
    /// Weight for semantic similarity (0.0 - 1.0).
    pub semantic_weight: f32,
    /// Weight for keyword similarity (0.0 - 1.0).
    pub keyword_weight: f32,
    /// Minimum similarity score threshold.
    pub min_score: f32,
}

impl HybridRetrievalConfig {
    /// Create with default weights (70% semantic, 30% keyword).
    pub fn default_weights() -> Self {
        Self {
            semantic_weight: 0.7,
            keyword_weight: 0.3,
            min_score: 0.1,
        }
    }

    /// Validate that weights sum to approximately 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.semantic_weight + self.keyword_weight;
        if (sum - 1.0).abs() > 0.01 {
            return Err(format!("Weights must sum to 1.0, got {}", sum));
        }
        if self.semantic_weight < 0.0 || self.keyword_weight < 0.0 {
            return Err("Weights must be non-negative".to_string());
        }
        Ok(())
    }
}

impl Default for HybridRetrievalConfig {
    fn default() -> Self {
        Self::default_weights()
    }
}

/// Hybrid retrieval engine combining semantic and keyword search.
///
/// This engine performs both semantic (embedding-based) and keyword (BM25-like)
/// searches and combines their scores using configurable weights.
pub struct HybridRetrievalEngine {
    config: HybridRetrievalConfig,
}

impl HybridRetrievalEngine {
    /// Create a new hybrid retrieval engine with configuration.
    pub fn new(config: HybridRetrievalConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self {
            config: HybridRetrievalConfig::default_weights(),
        }
    }

    /// Compute a hybrid score from semantic and keyword scores.
    pub fn compute_hybrid_score(&self, semantic_score: f32, keyword_score: f32) -> f32 {
        (semantic_score * self.config.semantic_weight)
            + (keyword_score * self.config.keyword_weight)
    }

    /// Normalize scores to 0.0 - 1.0 range.
    #[allow(dead_code)]
    fn normalize_score(score: f32) -> f32 {
        score.clamp(0.0, 1.0)
    }

    /// Combine semantic and keyword retrieval results.
    pub fn combine_results(
        &self,
        semantic_results: &SearchResultSet,
        keyword_results: &SearchResultSet,
        limit: usize,
    ) -> SearchResultSet {
        use std::collections::HashMap;

        // If one side is empty, fallback to the other side
        if semantic_results.is_empty() && !keyword_results.is_empty() {
            return keyword_results.clone();
        }
        if keyword_results.is_empty() && !semantic_results.is_empty() {
            return semantic_results.clone();
        }

        let mut map: HashMap<String, (f32, f32, SearchResult)> = HashMap::new();

        for sr in &semantic_results.results {
            map.insert(sr.vector_id.clone(), (sr.similarity_score, 0.0, sr.clone()));
        }

        for kr in &keyword_results.results {
            map.entry(kr.vector_id.clone())
                .and_modify(|entry| entry.1 = kr.similarity_score)
                .or_insert((0.0, kr.similarity_score, kr.clone()));
        }

        // Compute hybrid score and merge metadata/content
        let fused: Vec<SearchResult> = map
            .into_iter()
            .map(|(_id, (sem, key, mut sr))| {
                let hybrid = self.compute_hybrid_score(sem, key).clamp(0.0, 1.0);
                // Merge metadata: prefer entries already in sr, otherwise nothing to do
                sr.similarity_score = hybrid;
                sr
            })
            .collect();

        // Deduplicate by document id (keep highest score per document)
        let mut by_doc: HashMap<String, SearchResult> = HashMap::new();
        for r in fused.into_iter() {
            let doc_id = r.metadata.get("document_id").cloned().unwrap_or_default();
            by_doc
                .entry(doc_id)
                .and_modify(|existing| {
                    if r.similarity_score > existing.similarity_score {
                        *existing = r.clone();
                    }
                })
                .or_insert(r);
        }

        let mut results: Vec<SearchResult> = by_doc.into_values().collect();

        // Sort by hybrid score desc
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total = results.len();
        results.truncate(limit);

        // Build combined query string
        let query = format!("{} + {}", semantic_results.query, keyword_results.query);

        SearchResultSet::new(results, query, total, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_config_validation() {
        let config = HybridRetrievalConfig::default_weights();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_hybrid_score_computation() {
        let config = HybridRetrievalConfig::default_weights();
        let engine = HybridRetrievalEngine::new(config).unwrap();
        let score = engine.compute_hybrid_score(1.0, 0.5);
        assert!((score - 0.85).abs() < 0.01); // 0.7 * 1.0 + 0.3 * 0.5
    }

    #[test]
    fn test_invalid_weights() {
        let config = HybridRetrievalConfig {
            semantic_weight: 0.5,
            keyword_weight: 0.3,
            min_score: 0.1,
        };
        assert!(HybridRetrievalEngine::new(config).is_err());
    }
}
