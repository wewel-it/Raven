//! Embedding backend implementations and registry.
//!
//! This module provides multiple embedding backends that can be
//! selected dynamically through the BackendRegistry.

pub mod bge;
pub mod e5;
pub mod local;
pub mod nomic;
pub mod qwen;
pub mod registry;

#[path = "trait.rs"]
pub mod trait_impl;

pub use bge::BgeBackend;
pub use e5::E5Backend;
pub use local::LocalHashBackend;
pub use nomic::NomicBackend;
pub use qwen::QwenBackend;
pub use registry::{BackendConfig, BackendRegistry, ProviderInfo};
pub use trait_impl::{BackendError, BackendMetadata, BackendResult, EmbeddingBackend};

pub use trait_impl::EmbeddingBackend as Backend;
