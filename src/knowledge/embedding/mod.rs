//! Embedding and semantic search infrastructure.
//!
//! This module provides production-grade embedding generation, vector indexing,
//! and semantic search capabilities with multiple backend support.

pub mod backend;
pub mod batching;
pub mod cache;
pub mod engine;
pub mod metrics;
pub mod model;
pub mod normalize;
pub mod retrieval;
pub mod service;
pub mod similarity;
pub mod similarity_new;
pub mod tokenizer;
pub mod vector;

pub use cache::{EmbeddingCache, EmbeddingCacheStats};
pub use engine::{EmbeddingEngine, EmbeddingError, EmbeddingOutput, LocalEmbeddingEngine};
pub use model::{EmbeddingConfig, TfidfEmbeddingModel};
pub use retrieval::{
    ContextBuilder, HybridRetrievalConfig, HybridRetrievalEngine, SemanticSearchEngine,
};
pub use service::{EmbeddingService, EmbeddingServiceConfig};
pub use similarity::{
    CosineSimilarity, DotProductSimilarity, EuclideanDistanceSimilarity, SimilarityEngine,
    SimilarityMetric, SimilarityMetricType,
};
pub use tokenizer::SimpleTokenizer;
pub use vector::{
    DenseVector, MetadataStore, SearchResult, SearchResultSet, VectorIndex, VectorMetadata,
    VectorStorage,
};

// Re-export backend modules
pub use backend::{
    BackendConfig, BackendRegistry, BgeBackend, E5Backend, EmbeddingBackend, LocalHashBackend,
    NomicBackend, QwenBackend,
};
