use crate::knowledge::chunk::Chunk;
use crate::knowledge::document::Document;
use crate::knowledge::errors::{KnowledgeError, KnowledgeResult};
use crate::knowledge::traits::KnowledgeStorage;
use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory repository for knowledge documents and chunks.
#[derive(Debug, Default)]
pub struct InMemoryKnowledgeStorage {
    documents: Mutex<HashMap<String, Document>>,
    chunks: Mutex<HashMap<String, Vec<Chunk>>>,
}

impl InMemoryKnowledgeStorage {
    pub fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
            chunks: Mutex::new(HashMap::new()),
        }
    }
}

impl KnowledgeStorage for InMemoryKnowledgeStorage {
    fn save_document(&self, document: Document) -> KnowledgeResult<()> {
        let mut guard = self.documents.lock().map_err(|e| {
            KnowledgeError::Storage(format!("document storage lock poisoned: {}", e))
        })?;
        guard.insert(document.id().to_string(), document);
        Ok(())
    }

    fn save_chunks(&self, chunks: Vec<Chunk>) -> KnowledgeResult<()> {
        if let Some(first) = chunks.first() {
            let document_id = first.document_id().to_string();
            let mut guard = self.chunks.lock().map_err(|e| {
                KnowledgeError::Storage(format!("chunk storage lock poisoned: {}", e))
            })?;
            guard.insert(document_id, chunks);
            Ok(())
        } else {
            Err(KnowledgeError::Storage(
                "no chunks provided for storage".into(),
            ))
        }
    }

    fn get_document(&self, document_id: &str) -> KnowledgeResult<Option<Document>> {
        let guard = self.documents.lock().map_err(|e| {
            KnowledgeError::Storage(format!("document storage lock poisoned: {}", e))
        })?;
        Ok(guard.get(document_id).cloned())
    }

    fn remove_document(&self, document_id: &str) -> KnowledgeResult<()> {
        let mut docs = self.documents.lock().map_err(|e| {
            KnowledgeError::Storage(format!("document storage lock poisoned: {}", e))
        })?;
        docs.remove(document_id);
        let mut chunks = self
            .chunks
            .lock()
            .map_err(|e| KnowledgeError::Storage(format!("chunk storage lock poisoned: {}", e)))?;
        chunks.remove(document_id);
        Ok(())
    }

    fn list_documents(&self) -> KnowledgeResult<Vec<Document>> {
        let guard = self.documents.lock().map_err(|e| {
            KnowledgeError::Storage(format!("document storage lock poisoned: {}", e))
        })?;
        Ok(guard.values().cloned().collect())
    }

    fn list_chunks(&self, document_id: &str) -> KnowledgeResult<Vec<Chunk>> {
        let guard = self
            .chunks
            .lock()
            .map_err(|e| KnowledgeError::Storage(format!("chunk storage lock poisoned: {}", e)))?;
        Ok(guard.get(document_id).cloned().unwrap_or_default())
    }

    fn clear(&self) -> KnowledgeResult<()> {
        let mut docs = self.documents.lock().map_err(|e| {
            KnowledgeError::Storage(format!("document storage lock poisoned: {}", e))
        })?;
        docs.clear();
        let mut chunks = self
            .chunks
            .lock()
            .map_err(|e| KnowledgeError::Storage(format!("chunk storage lock poisoned: {}", e)))?;
        chunks.clear();
        Ok(())
    }
}
