use crate::knowledge::chunker::FixedChunker;
use crate::knowledge::hash::Blake3HashEngine;
use crate::knowledge::loader::FileLoader;
use crate::knowledge::pipeline::KnowledgePipeline;
use crate::knowledge::storage::InMemoryKnowledgeStorage;
use crate::knowledge::traits::{
    Chunker, DocumentLoader, DocumentValidator, HashEngine, KnowledgeStorage,
};
use crate::knowledge::validator::FileValidator;

/// Builder for the knowledge processing pipeline.
pub struct KnowledgePipelineBuilder {
    validator: Option<Box<dyn DocumentValidator>>,
    loader: Option<Box<dyn DocumentLoader>>,
    chunker: Option<Box<dyn Chunker>>,
    hash_engine: Option<Box<dyn HashEngine>>,
    storage: Option<Box<dyn KnowledgeStorage>>,
}

impl KnowledgePipelineBuilder {
    pub fn new() -> Self {
        Self {
            validator: None,
            loader: None,
            chunker: None,
            hash_engine: None,
            storage: None,
        }
    }

    pub fn with_validator(mut self, validator: Box<dyn DocumentValidator>) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn with_loader(mut self, loader: Box<dyn DocumentLoader>) -> Self {
        self.loader = Some(loader);
        self
    }

    pub fn with_chunker(mut self, chunker: Box<dyn Chunker>) -> Self {
        self.chunker = Some(chunker);
        self
    }

    pub fn with_hash_engine(mut self, hash_engine: Box<dyn HashEngine>) -> Self {
        self.hash_engine = Some(hash_engine);
        self
    }

    pub fn with_storage(mut self, storage: Box<dyn KnowledgeStorage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn build(self) -> KnowledgePipeline {
        KnowledgePipeline::new(
            self.validator
                .unwrap_or_else(|| Box::new(FileValidator::new())),
            self.loader.unwrap_or_else(|| Box::new(FileLoader::new())),
            self.chunker
                .unwrap_or_else(|| Box::new(FixedChunker::new(1024))),
            self.hash_engine
                .unwrap_or_else(|| Box::new(Blake3HashEngine::new())),
            self.storage
                .unwrap_or_else(|| Box::new(InMemoryKnowledgeStorage::new())),
        )
    }
}
