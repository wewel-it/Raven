use crate::knowledge::metadata::DocumentMetadata;

/// Document chunk produced by deterministic chunking.
#[derive(Clone, Debug)]
pub struct Chunk {
    id: String,
    document_id: String,
    sequence: usize,
    content: String,
    metadata: DocumentMetadata,
    hash: String,
}

impl Chunk {
    pub fn new(
        id: String,
        document_id: String,
        sequence: usize,
        content: String,
        metadata: DocumentMetadata,
        hash: String,
    ) -> Self {
        Self {
            id,
            document_id,
            sequence,
            content,
            metadata,
            hash,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    pub fn sequence(&self) -> usize {
        self.sequence
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }
}
