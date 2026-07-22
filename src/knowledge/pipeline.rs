use crate::knowledge::chunk::Chunk;
use crate::knowledge::document::Document;
use crate::knowledge::errors::{KnowledgeError, KnowledgeResult};
use crate::knowledge::traits::{
    Chunker, DocumentLoader, DocumentValidator, HashEngine, KnowledgeStorage,
};
use std::path::Path;

/// Modular processing pipeline for knowledge documents.
pub struct KnowledgePipeline {
    validator: Box<dyn DocumentValidator>,
    loader: Box<dyn DocumentLoader>,
    chunker: Box<dyn Chunker>,
    hash_engine: Box<dyn HashEngine>,
    storage: Box<dyn KnowledgeStorage>,
}

impl KnowledgePipeline {
    pub fn new(
        validator: Box<dyn DocumentValidator>,
        loader: Box<dyn DocumentLoader>,
        chunker: Box<dyn Chunker>,
        hash_engine: Box<dyn HashEngine>,
        storage: Box<dyn KnowledgeStorage>,
    ) -> Self {
        Self {
            validator,
            loader,
            chunker,
            hash_engine,
            storage,
        }
    }

    pub fn process_file(&self, path: &Path) -> KnowledgeResult<String> {
        self.validator.validate(path)?;
        let mut document = self.loader.load(path)?;
        let content_hash = self.hash_engine.hash(document.content().as_bytes());
        let metadata = document.metadata().clone();
        let metadata = crate::knowledge::metadata::DocumentMetadata::new(
            metadata.title().to_string(),
            metadata.author().map(|s| s.to_string()),
            metadata.language().to_string(),
            metadata.category().to_string(),
            metadata.topic().map(|s| s.to_string()),
            metadata.tags().to_vec(),
            metadata.difficulty().to_string(),
            metadata.version().to_string(),
            metadata.source().to_string(),
            content_hash.clone(),
            metadata.size(),
            metadata.created_at(),
            metadata.updated_at(),
        );
        document = Document::new(
            document.id().to_string(),
            document.path().to_path_buf(),
            document.title().to_string(),
            document.language().to_string(),
            document.tags().to_vec(),
            document.source().to_string(),
            metadata,
            document.content().to_string(),
        );

        let chunks = self.chunker.chunk(&document)?;
        let chunks = chunks
            .into_iter()
            .map(|chunk| {
                let chunk_hash = self.hash_engine.hash(chunk.content().as_bytes());
                let chunk = crate::knowledge::chunk::Chunk::new(
                    chunk.id().to_string(),
                    chunk.document_id().to_string(),
                    chunk.sequence(),
                    chunk.content().to_string(),
                    chunk.metadata().clone(),
                    chunk_hash,
                );
                chunk
            })
            .collect::<Vec<Chunk>>();

        self.storage.save_document(document.clone())?;
        self.storage.save_chunks(chunks)?;
        Ok(document.id().to_string())
    }

    pub fn process_directory(&self, root: &Path) -> KnowledgeResult<Vec<String>> {
        let mut processed = Vec::new();

        fn walk(
            path: &Path,
            pipeline: &KnowledgePipeline,
            processed: &mut Vec<String>,
        ) -> KnowledgeResult<()> {
            for entry in path
                .read_dir()
                .map_err(|err| KnowledgeError::Io(err.to_string()))?
            {
                let entry = entry.map_err(|err| KnowledgeError::Io(err.to_string()))?;
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, pipeline, processed)?;
                } else if path.is_file() {
                    let extension = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default();
                    if extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("txt")
                    {
                        let id = pipeline.process_file(&path)?;
                        processed.push(id);
                    }
                }
            }
            Ok(())
        }

        walk(root, self, &mut processed)?;
        Ok(processed)
    }

    pub fn storage(&self) -> &dyn KnowledgeStorage {
        self.storage.as_ref()
    }
}
