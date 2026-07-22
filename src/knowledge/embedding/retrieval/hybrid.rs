//! Hybrid retrieval combining semantic and keyword search.

use crate::knowledge::chunk::Chunk;
use crate::knowledge::retrieval::RetrievalResult;

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
            return Err(format!(
                "Weights must sum to 1.0, got {}",
                sum
            ));
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
    fn normalize_score(score: f32) -> f32 {
        score.max(0.0).min(1.0)
    }

    /// Combine semantic and keyword retrieval results.
    pub fn combine_results(
        &self,
        semantic_results: RetrievalResult,
        keyword_results: RetrievalResult,
        limit: usize,
    ) -> RetrievalResult {
        // Create a map of chunk IDs to their scores
        use std::collections::HashMap;

        let mut combined_scores: HashMap<String, (f32, Chunk, bool)> = HashMap::new();

        // Add semantic results (with document)
        for chunk in &semantic_results.top_chunks {
            let score = semantic_results
                .top_chunks
                .iter()
                .position(|c| c.id() == chunk.id())
                .map(|pos| {
                    // Convert position to score (1.0 for best, decreasing)
                    1.0 / (1.0 + pos as f32 * 0.1)
                })
                .unwrap_or(0.5);

            combined_scores
                .entry(chunk.id().to_string())
                .and_modify(|(s, _, has_semantic)| {
                    *s = s.max(
                        self.compute_hybrid_score(
                            Self::normalize_score(score),
                            Self::normalize_score(*s),
                        ),
                    );
                    *has_semantic = true;
                })
                .or_insert((
                    self.compute_hybrid_score(Self::normalize_score(score), 0.0),
                    chunk.clone(),
                    true,
                ));
        }

        // Add keyword results (with document)
        for chunk in &keyword_results.top_chunks {
            let score = keyword_results
                .top_chunks
                .iter()
                .position(|c| c.id() == chunk.id())
                .map(|pos| {
                    // Convert position to score (1.0 for best, decreasing)
                    1.0 / (1.0 + pos as f32 * 0.1)
                })
                .unwrap_or(0.3);

            combined_scores
                .entry(chunk.id().to_string())
                .and_modify(|(s, _, _)| {
                    *s = self.compute_hybrid_score(
                        Self::normalize_score(*s),
                        Self::normalize_score(score),
                    );
                })
                .or_insert((
                    self.compute_hybrid_score(0.0, Self::normalize_score(score)),
                    chunk.clone(),
                    false,
                ));
        }

        // Sort by combined score
        let mut sorted: Vec<_> = combined_scores.into_values().collect();
        sorted.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Extract top-k chunks
        let top_chunks: Vec<Chunk> = sorted
            .into_iter()
            .take(limit)
            .map(|(_, chunk, _)| chunk)
            .collect();

        let query = format!("{} + {}", semantic_results.query, keyword_results.query);

        RetrievalResult::new(
            query,
            vec![], // Documents would need to be reconstructed from chunks
            top_chunks,
            std::cmp::max(
                semantic_results.candidate_count,
                keyword_results.candidate_count,
            ),
        )
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
