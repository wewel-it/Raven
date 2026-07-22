//! Re-ranking and post-processing of search results.

use crate::knowledge::embedding::vector::search::{SearchResult, SearchResultSet};

/// Re-ranking strategy for search results.
#[derive(Debug, Clone, Copy)]
pub enum RerankingStrategy {
    /// No re-ranking, keep original order.
    None,
    /// Re-rank by relevance score (higher is better).
    ByScore,
    /// Re-rank by content length (prefer longer, more detailed results).
    ByLength,
    /// Re-rank by source diversity (prefer results from different sources).
    ByDiversity,
}

/// A reranker for improving search result quality.
pub struct SearchResultReranker {
    strategy: RerankingStrategy,
}

impl SearchResultReranker {
    /// Create a new reranker with the specified strategy.
    pub fn new(strategy: RerankingStrategy) -> Self {
        Self { strategy }
    }

    /// Re-rank a search result set.
    pub fn rerank(&self, results: SearchResultSet) -> SearchResultSet {
        match self.strategy {
            RerankingStrategy::None => results,
            RerankingStrategy::ByScore => self.rerank_by_score(results),
            RerankingStrategy::ByLength => self.rerank_by_length(results),
            RerankingStrategy::ByDiversity => self.rerank_by_diversity(results),
        }
    }

    /// Re-rank by similarity score (already sorted, but normalize).
    fn rerank_by_score(&self, mut results: SearchResultSet) -> SearchResultSet {
        results.results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Re-rank by content length (prefer more detailed).
    fn rerank_by_length(&self, mut results: SearchResultSet) -> SearchResultSet {
        results.results.sort_by(|a, b| {
            // Sort by length (descending) but also consider score
            let a_score = a.similarity_score * (1.0 + (a.content.len() as f32 / 1000.0).min(1.0));
            let b_score = b.similarity_score * (1.0 + (b.content.len() as f32 / 1000.0).min(1.0));
            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Re-rank by diversity (prefer results from different modules/sources).
    fn rerank_by_diversity(&self, mut results: SearchResultSet) -> SearchResultSet {
        let mut seen_sources = std::collections::HashSet::new();
        let mut diversity_scores: Vec<(usize, f32)> = Vec::new();

        for (idx, result) in results.results.iter().enumerate() {
            let module = result
                .metadata
                .get("module")
                .map(|m| m.as_str())
                .unwrap_or("unknown");
            let diversity_bonus = if seen_sources.insert(module) { 0.1 } else { 0.0 };
            let score = result.similarity_score + diversity_bonus;
            diversity_scores.push((idx, score));
        }

        diversity_scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let reranked = diversity_scores
            .into_iter()
            .map(|(idx, _)| results.results[idx].clone())
            .collect();

        SearchResultSet::new(
            reranked,
            results.query,
            results.total_searched,
            results.total_candidates,
        )
    }
}

impl Default for SearchResultReranker {
    fn default() -> Self {
        Self::new(RerankingStrategy::ByScore)
    }
}

/// Context builder for constructing context from search results.
pub struct ContextBuilder;

impl ContextBuilder {
    /// Build a context string from search results.
    pub fn build_context(
        results: &SearchResultSet,
        max_tokens: usize,
    ) -> String {
        let mut context = String::new();
        let mut tokens_used = 0;

        for result in results.top_k(5) {
            let entry = format!(
                "## {} (similarity: {:.2}%)\n{}\n\n",
                result.vector_id,
                result.similarity_score * 100.0,
                result.content
            );
            let entry_tokens = entry.split_whitespace().count();

            if tokens_used + entry_tokens <= max_tokens {
                context.push_str(&entry);
                tokens_used += entry_tokens;
            } else {
                break;
            }
        }

        context
    }

    /// Build a compact context (just content, no metadata).
    pub fn build_compact_context(results: &SearchResultSet) -> String {
        results
            .top_k(3)
            .iter()
            .map(|r| r.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    /// Deduplicate and merge results before context building.
    pub fn prepare_context(results: SearchResultSet) -> SearchResultSet {
        results.deduplicate_by_document()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_result(id: &str, score: f32, content: &str) -> SearchResult {
        SearchResult {
            vector_id: id.to_string(),
            similarity_score: score,
            content: content.to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_rerank_by_score() {
        let results = SearchResultSet::new(
            vec![
                make_result("v1", 0.5, "content 1"),
                make_result("v2", 0.9, "content 2"),
                make_result("v3", 0.7, "content 3"),
            ],
            "query".to_string(),
            3,
            10,
        );

        let reranker = SearchResultReranker::new(RerankingStrategy::ByScore);
        let reranked = reranker.rerank(results);
        assert_eq!(reranked.results[0].similarity_score, 0.9);
    }

    #[test]
    fn test_build_context() {
        let results = SearchResultSet::new(
            vec![
                make_result("v1", 0.9, "This is a long context that contains information"),
                make_result("v2", 0.8, "Another piece of context"),
            ],
            "query".to_string(),
            2,
            10,
        );

        let context = ContextBuilder::build_context(&results, 100);
        assert!(!context.is_empty());
        assert!(context.contains("This is a long context"));
    }

    #[test]
    fn test_build_compact_context() {
        let results = SearchResultSet::new(
            vec![
                make_result("v1", 0.9, "content 1"),
                make_result("v2", 0.8, "content 2"),
                make_result("v3", 0.7, "content 3"),
            ],
            "query".to_string(),
            3,
            10,
        );

        let context = ContextBuilder::build_compact_context(&results);
        assert!(context.contains("content 1"));
        assert!(context.contains("content 2"));
    }
}
