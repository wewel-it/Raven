use crate::knowledge::context::KnowledgeContext;
use crate::knowledge::document::Document;
use crate::knowledge::errors::KnowledgeResult;
use crate::knowledge::metadata::DocumentMetadata;
use crate::knowledge::pipeline::KnowledgePipeline;
use crate::knowledge::retrieval::{KnowledgeRetrievalEngine, SemanticRetrievalEngine};
use crate::knowledge::traits::KnowledgeManager;
use chrono::Utc;
use std::path::{Path, PathBuf};

/// Façade for the Knowledge Library.
pub struct KnowledgeManagerImpl {
    pipeline: KnowledgePipeline,
    retrieval_engine: Box<dyn KnowledgeRetrievalEngine>,
}

impl KnowledgeManagerImpl {
    pub fn new(
        pipeline: KnowledgePipeline,
        retrieval_engine: Box<dyn KnowledgeRetrievalEngine>,
    ) -> Self {
        Self {
            pipeline,
            retrieval_engine,
        }
    }

    pub fn new_with_default_engine(pipeline: KnowledgePipeline) -> Self {
        Self::new(pipeline, Box::new(SemanticRetrievalEngine::new()))
    }
}

impl KnowledgeManager for KnowledgeManagerImpl {
    fn add_document(&self, path: &Path) -> KnowledgeResult<String> {
        self.pipeline.process_file(path)
    }

    fn remove_document(&self, document_id: &str) -> KnowledgeResult<()> {
        self.pipeline.storage().remove_document(document_id)
    }

    fn update_document(&self, path: &Path) -> KnowledgeResult<String> {
        let id = self.pipeline.process_file(path)?;
        Ok(id)
    }

    fn list_documents(&self) -> KnowledgeResult<Vec<Document>> {
        self.pipeline.storage().list_documents()
    }

    fn load_document(&self, document_id: &str) -> KnowledgeResult<Option<Document>> {
        self.pipeline.storage().get_document(document_id)
    }

    fn process_document(&self, path: &Path) -> KnowledgeResult<String> {
        self.pipeline.process_file(path)
    }

    fn process_directory(&self, root: &Path) -> KnowledgeResult<Vec<String>> {
        self.pipeline.process_directory(root)
    }

    fn rebuild_library(&self, root: &Path) -> KnowledgeResult<Vec<String>> {
        self.pipeline.storage().clear()?;
        self.pipeline.process_directory(root)
    }

    fn retrieve(&self, query: &str, limit: usize) -> KnowledgeResult<KnowledgeContext> {
        let result = self
            .retrieval_engine
            .retrieve(self.pipeline.storage(), query, limit)?;

        Ok(KnowledgeContext::new(
            query.to_string(),
            result.top_documents,
            result.top_chunks,
            result.candidate_count,
        ))
    }

    fn store(&self, title: &str, content: &str, tags: &[String]) -> KnowledgeResult<String> {
        let now = Utc::now();
        let timestamp = now
            .timestamp_nanos_opt()
            .unwrap_or_else(|| now.timestamp() * 1_000_000_000);
        let id = format!("store:{}:{}", title, timestamp);
        let metadata = DocumentMetadata::new(
            title.to_string(),
            None,
            "text".to_string(),
            "reflection".to_string(),
            None,
            tags.to_vec(),
            "intermediate".to_string(),
            "1.0".to_string(),
            "local".to_string(),
            id.clone(),
            content.len() as u64,
            now,
            now,
        );
        let document = Document::new(
            id.clone(),
            PathBuf::from(""),
            title.to_string(),
            "text".to_string(),
            tags.to_vec(),
            "reflection".to_string(),
            metadata,
            content.to_string(),
        );
        self.pipeline.storage().save_document(document)?;
        Ok(id)
    }
}
