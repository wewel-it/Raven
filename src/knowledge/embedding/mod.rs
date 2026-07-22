//! Embedding and semantic search infrastructure.
//!
//! This module provides production-grade embedding generation, vector indexing,
//! and semantic search capabilities using TF-IDF embeddings with caching.

pub mod cache;
pub mod engine;
pub mod model;
pub mod retrieval;
pub mod similarity;
pub mod tokenizer;
pub mod vector;

pub use cache::{EmbeddingCache, EmbeddingCacheStats};
pub use engine::{EmbeddingEngine, EmbeddingError, EmbeddingOutput, LocalEmbeddingEngine};
pub use model::{EmbeddingConfig, TfidfEmbeddingModel};
pub use retrieval::{ContextBuilder, HybridRetrievalConfig, HybridRetrievalEngine, SemanticSearchEngine};
pub use similarity::{CosineSimilarity, DotProductSimilarity, EuclideanDistanceSimilarity, SimilarityEngine, SimilarityMetric, SimilarityMetricType};
pub use tokenizer::SimpleTokenizer;
pub use vector::{DenseVector, MetadataStore, SearchResult, SearchResultSet, VectorIndex, VectorMetadata, VectorStorage};
