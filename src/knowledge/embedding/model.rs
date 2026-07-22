//! Embedding model and TF-IDF implementation.

use crate::knowledge::embedding::similarity::SimilarityMetricType;
use crate::knowledge::embedding::tokenizer::SimpleTokenizer;
use crate::knowledge::embedding::vector::DenseVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for embedding model behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding dimension (number of features).
    pub dimension: usize,
    /// Maximum number of features (tokens) to use in the vocabulary.
    pub max_features: usize,
    /// Similarity metric to use.
    pub similarity_metric: SimilarityMetricType,
    /// Whether to use TF-IDF weighting (true) or simple TF (false).
    pub use_tfidf: bool,
    /// Minimum document frequency for a term to be included.
    pub min_df: usize,
    /// Maximum document frequency for a term (filters very common terms).
    pub max_df: f32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            max_features: 50000,
            similarity_metric: SimilarityMetricType::Cosine,
            use_tfidf: true,
            min_df: 1,
            max_df: 0.95,
        }
    }
}

impl EmbeddingConfig {
    /// Create a new config with custom dimension.
    pub fn with_dimension(dimension: usize) -> Self {
        Self {
            dimension,
            ..Default::default()
        }
    }
}

/// A TF-IDF based embedding model for text vectorization.
///
/// This is a production-grade, deterministic embedding that uses:
/// - Simple tokenization
/// - TF-IDF weighting (or simple TF)
/// - Feature hashing to map tokens to fixed dimensions
/// - L2 normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfidfEmbeddingModel {
    config: EmbeddingConfig,
    tokenizer: SimpleTokenizer,
    /// Token to index mapping (vocabulary).
    vocabulary: HashMap<String, usize>,
    /// IDF values for each term in vocabulary.
    idf: HashMap<String, f32>,
    /// Number of documents seen during IDF computation.
    num_documents: usize,
}

impl TfidfEmbeddingModel {
    /// Create a new TF-IDF embedding model with default configuration.
    pub fn new() -> Self {
        Self {
            config: EmbeddingConfig::default(),
            tokenizer: SimpleTokenizer::new(),
            vocabulary: HashMap::new(),
            idf: HashMap::new(),
            num_documents: 0,
        }
    }

    /// Create a new TF-IDF embedding model with custom configuration.
    pub fn with_config(config: EmbeddingConfig) -> Self {
        Self {
            config,
            tokenizer: SimpleTokenizer::new(),
            vocabulary: HashMap::new(),
            idf: HashMap::new(),
            num_documents: 0,
        }
    }

    /// Build the model's vocabulary and compute IDF values from documents.
    pub fn fit(&mut self, documents: &[&str]) -> Result<(), String> {
        self.num_documents = documents.len();
        if self.num_documents == 0 {
            return Err("Cannot fit model with empty documents".to_string());
        }

        // Collect all unique tokens and their document frequencies
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        for doc in documents {
            let tokens = self.tokenizer.tokenize_filtered(doc);
            let unique_tokens: std::collections::HashSet<_> = tokens.into_iter().collect();
            for token in unique_tokens {
                *doc_freq.entry(token).or_insert(0) += 1;
            }
        }

        // Filter tokens by min_df and max_df
        let max_df_count = (self.num_documents as f32 * self.config.max_df).ceil() as usize;
        let mut filtered_tokens: Vec<_> = doc_freq
            .into_iter()
            .filter(|(_, df)| *df >= self.config.min_df && *df <= max_df_count)
            .map(|(token, df)| (token, df))
            .collect();

        // Sort by frequency (most common first) and take top max_features
        filtered_tokens.sort_by(|a, b| b.1.cmp(&a.1));
        filtered_tokens.truncate(self.config.max_features);

        // Build vocabulary and compute IDF
        for (idx, (token, df)) in filtered_tokens.iter().enumerate() {
            self.vocabulary.insert(token.clone(), idx);
            let idf = ((self.num_documents as f32 / *df as f32) + 1.0).ln();
            self.idf.insert(token.clone(), idf);
        }

        Ok(())
    }

    /// Embed a single text using feature hashing and TF-IDF.
    pub fn embed(&self, text: &str) -> DenseVector {
        // Tokenize and compute term frequencies
        let tokens = self.tokenizer.tokenize_filtered(text);
        let tf = self.tokenizer.term_frequencies(&tokens);

        // Initialize vector
        let mut vector = vec![0.0; self.config.dimension];

        // Accumulate weighted token features
        for (token, count) in tf.iter() {
            if let Some(idf) = self.idf.get(token) {
                // Compute feature index via hashing
                let hash = self.hash_token(token);
                let idx = hash % self.config.dimension;

                // Compute TF (simple or log-scaled)
                let term_tf = if self.config.use_tfidf {
                    (*count as f32).log2() + 1.0
                } else {
                    *count as f32
                };

                // Weight by IDF if enabled
                let weight = if self.config.use_tfidf {
                    term_tf * idf
                } else {
                    term_tf
                };

                vector[idx] += weight;
            }
        }

        // Create vector and normalize
        let mut result = DenseVector::new(vector);
        result.normalize_inplace();
        result
    }

    /// Embed multiple texts (documents or chunks).
    pub fn embed_batch(&self, texts: &[&str]) -> Vec<DenseVector> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    /// Hash a token to a deterministic value.
    ///
    /// Uses a simple multiplicative hash to ensure consistency.
    fn hash_token(&self, token: &str) -> usize {
        let mut hash: usize = 5381;
        for c in token.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as usize);
        }
        hash
    }

    /// Get the embedding dimension.
    pub fn dimension(&self) -> usize {
        self.config.dimension
    }

    /// Get the vocabulary size.
    pub fn vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }

    /// Get the IDF value for a term (0.0 if not in vocabulary).
    pub fn get_idf(&self, token: &str) -> f32 {
        self.idf.get(token).copied().unwrap_or(0.0)
    }

    /// Check if a token is in the vocabulary.
    pub fn has_token(&self, token: &str) -> bool {
        self.vocabulary.contains_key(token)
    }
}

impl Default for TfidfEmbeddingModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_creation() {
        let model = TfidfEmbeddingModel::new();
        assert_eq!(model.dimension(), 768);
    }

    #[test]
    fn test_fit_and_embed() {
        let documents = vec![
            "machine learning is great",
            "deep learning is powerful",
            "natural language processing",
        ];
        let mut model = TfidfEmbeddingModel::new();
        assert!(model.fit(&documents).is_ok());
        assert!(model.vocabulary_size() > 0);

        let embedding = model.embed("machine learning");
        assert_eq!(embedding.dimension(), 768);
        assert!((embedding.l2_norm() - 1.0).abs() < 1e-6); // Should be normalized
    }

    #[test]
    fn test_embed_batch() {
        let documents = vec![
            "machine learning is great",
            "deep learning is powerful",
            "natural language processing",
        ];
        let mut model = TfidfEmbeddingModel::new();
        let _ = model.fit(&documents);

        let embeddings = model.embed_batch(&["machine learning", "deep learning"]);
        assert_eq!(embeddings.len(), 2);
        for emb in embeddings {
            assert_eq!(emb.dimension(), 768);
        }
    }

    #[test]
    fn test_deterministic_embedding() {
        let mut model = TfidfEmbeddingModel::new();
        let documents = vec!["test document", "another document"];
        let _ = model.fit(&documents);

        let emb1 = model.embed("test");
        let emb2 = model.embed("test");
        assert_eq!(emb1.data(), emb2.data());
    }

    #[test]
    fn test_normalization() {
        let mut model = TfidfEmbeddingModel::new();
        let documents = vec!["hello world", "foo bar baz"];
        let _ = model.fit(&documents);

        let embedding = model.embed("hello world test");
        let norm = embedding.l2_norm();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_custom_config() {
        let config = EmbeddingConfig::with_dimension(128);
        let mut model = TfidfEmbeddingModel::with_config(config);
        let documents = vec!["test document"];
        let _ = model.fit(&documents);

        let embedding = model.embed("test");
        assert_eq!(embedding.dimension(), 128);
    }
}
