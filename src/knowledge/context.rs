use crate::knowledge::chunk::Chunk;
use crate::knowledge::document::Document;

/// Retrieved knowledge data transported through runtime and workflow context.
#[derive(Clone, Debug)]
pub struct KnowledgeContext {
    pub query: String,
    pub documents: Vec<Document>,
    pub chunks: Vec<Chunk>,
    pub document_count: usize,
    pub chunk_count: usize,
    pub candidate_count: usize,
}

impl KnowledgeContext {
    pub fn new(
        query: String,
        documents: Vec<Document>,
        chunks: Vec<Chunk>,
        candidate_count: usize,
    ) -> Self {
        let document_count = documents.len();
        let chunk_count = chunks.len();
        Self {
            query,
            documents,
            chunks,
            document_count,
            chunk_count,
            candidate_count,
        }
    }

    pub fn summary(&self) -> String {
        let titles = self
            .documents
            .iter()
            .map(|doc| doc.title().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "query='{}' documents={} chunks={} candidates={} titles=[{}]",
            self.query, self.document_count, self.chunk_count, self.candidate_count, titles
        )
    }
}
