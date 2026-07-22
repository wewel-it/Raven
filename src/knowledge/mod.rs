//! Knowledge Library foundation for Raven.
//!
//! The Knowledge Library is a modular repository for document ingestion,
//! validation, chunking, hashing, and storage. It is intentionally isolated
//! from LLM, RAG, and search backends.

pub mod builder;
pub mod chunk;
pub mod chunker;
pub mod context;
pub mod document;
pub mod errors;
pub mod hash;
pub mod loader;
pub mod manager;
pub mod metadata;
pub mod pipeline;
pub mod retrieval;
pub mod storage;
pub mod traits;
pub mod validator;

pub use builder::KnowledgePipelineBuilder;
pub use context::KnowledgeContext;
pub use errors::KnowledgeResult;
pub use manager::KnowledgeManagerImpl;
pub use retrieval::{KnowledgeRetrievalEngine, RetrievalResult, SemanticRetrievalEngine};
pub use traits::KnowledgeManager;
