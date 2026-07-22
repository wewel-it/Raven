use crate::knowledge::metadata::DocumentMetadata;
use std::path::{Path, PathBuf};

/// Document model representing a knowledge artifact.
#[derive(Clone, Debug)]
pub struct Document {
    id: String,
    path: PathBuf,
    title: String,
    language: String,
    tags: Vec<String>,
    source: String,
    metadata: DocumentMetadata,
    content: String,
}

impl Document {
    pub fn new(
        id: String,
        path: PathBuf,
        title: String,
        language: String,
        tags: Vec<String>,
        source: String,
        metadata: DocumentMetadata,
        content: String,
    ) -> Self {
        Self {
            id,
            path,
            title,
            language,
            tags,
            source,
            metadata,
            content,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}
