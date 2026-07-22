use crate::knowledge::errors::KnowledgeResult;
use crate::knowledge::{chunk::Chunk, document::Document};
use std::path::Path;

/// The gateway trait for the Knowledge Library façade.
pub trait KnowledgeManager: Send + Sync {
    fn add_document(&self, path: &Path) -> KnowledgeResult<String>;
    fn remove_document(&self, document_id: &str) -> KnowledgeResult<()>;
    fn update_document(&self, path: &Path) -> KnowledgeResult<String>;
    fn list_documents(&self) -> KnowledgeResult<Vec<Document>>;
    fn load_document(&self, document_id: &str) -> KnowledgeResult<Option<Document>>;
    fn process_document(&self, path: &Path) -> KnowledgeResult<String>;
    fn process_directory(&self, root: &Path) -> KnowledgeResult<Vec<String>>;
    fn rebuild_library(&self, root: &Path) -> KnowledgeResult<Vec<String>>;
    fn retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> KnowledgeResult<crate::knowledge::KnowledgeContext>;
    fn store(&self, title: &str, content: &str, tags: &[String]) -> KnowledgeResult<String>;
}

/// Validator trait for document inputs.
pub trait DocumentValidator: Send + Sync {
    fn validate(&self, path: &Path) -> KnowledgeResult<()>;
}

/// Loader trait to build documents from files.
pub trait DocumentLoader: Send + Sync {
    fn load(&self, path: &Path) -> KnowledgeResult<Document>;
}

/// Chunker trait for deterministic chunk generation.
pub trait Chunker: Send + Sync {
    fn chunk(&self, document: &Document) -> KnowledgeResult<Vec<Chunk>>;
}

/// Trait for hashing content.
pub trait HashEngine: Send + Sync {
    fn hash(&self, data: &[u8]) -> String;
}

/// Storage abstraction for knowledge artifacts.
pub trait KnowledgeStorage: Send + Sync {
    fn save_document(&self, document: Document) -> KnowledgeResult<()>;
    fn save_chunks(&self, chunks: Vec<Chunk>) -> KnowledgeResult<()>;
    fn get_document(&self, document_id: &str) -> KnowledgeResult<Option<Document>>;
    fn remove_document(&self, document_id: &str) -> KnowledgeResult<()>;
    fn list_documents(&self) -> KnowledgeResult<Vec<Document>>;
    fn list_chunks(&self, document_id: &str) -> KnowledgeResult<Vec<Chunk>>;
    fn clear(&self) -> KnowledgeResult<()>;
}
