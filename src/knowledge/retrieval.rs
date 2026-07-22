use crate::knowledge::chunk::Chunk;
use crate::knowledge::document::Document;
use crate::knowledge::errors::{KnowledgeError, KnowledgeResult};
use crate::knowledge::traits::KnowledgeStorage;
use regex::Regex;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

/// Result of a semantic retrieval request.
#[derive(Clone, Debug)]
pub struct RetrievalResult {
    pub query: String,
    pub top_documents: Vec<Document>,
    pub top_chunks: Vec<Chunk>,
    pub candidate_count: usize,
    pub document_count: usize,
    pub chunk_count: usize,
}

impl RetrievalResult {
    pub fn new(
        query: String,
        top_documents: Vec<Document>,
        top_chunks: Vec<Chunk>,
        candidate_count: usize,
    ) -> Self {
        let document_count = top_documents.len();
        let chunk_count = top_chunks.len();
        Self {
            query,
            top_documents,
            top_chunks,
            candidate_count,
            document_count,
            chunk_count,
        }
    }
}

/// Trait for knowledge retrieval engines.
pub trait KnowledgeRetrievalEngine: Send + Sync {
    fn retrieve(
        &self,
        storage: &dyn KnowledgeStorage,
        query: &str,
        limit: usize,
    ) -> KnowledgeResult<RetrievalResult>;
}

/// A semantic retrieval engine that scores documents and chunks by query overlap.
pub struct SemanticRetrievalEngine {
    normalizer: Regex,
}

impl SemanticRetrievalEngine {
    pub fn new() -> Self {
        Self {
            normalizer: Regex::new(r"[^\p{L}\p{N}\s]").unwrap(),
        }
    }

    fn normalize_text(&self, text: &str) -> String {
        let lower = text.nfkd().collect::<String>().to_lowercase();
        let cleaned = self.normalizer.replace_all(&lower, " ");
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|token| token.to_string())
            .collect()
    }

    fn token_set(&self, text: &str) -> HashSet<String> {
        self.tokenize(text).into_iter().collect()
    }

    fn score_text(&self, text: &str, query_tokens: &HashSet<String>) -> f64 {
        let tokens = self.tokenize(&self.normalize_text(text));
        let matches = tokens
            .iter()
            .filter(|token| query_tokens.contains(*token))
            .count() as f64;
        matches
    }

    fn score_chunk(&self, chunk: &Chunk, query_tokens: &HashSet<String>) -> f64 {
        let normalized = self.normalize_text(chunk.content());
        let tokens = self.tokenize(&normalized);
        if tokens.is_empty() {
            return 0.0;
        }

        let overlap = self
            .tokenize(&normalized)
            .iter()
            .filter(|token| query_tokens.contains(*token))
            .count() as f64;

        if overlap <= 0.0 {
            0.0
        } else {
            overlap / (tokens.len() as f64) * 8.0
        }
    }

    fn score_document(
        &self,
        document: &Document,
        chunks: &[Chunk],
        query_tokens: &HashSet<String>,
    ) -> f64 {
        let title_score = self.score_text(document.title(), query_tokens) * 4.0;
        let tag_score = document
            .tags()
            .iter()
            .map(|tag| self.score_text(tag, query_tokens) * 3.0)
            .sum::<f64>();
        let source_score = self.score_text(document.source(), query_tokens) * 2.0;
        let chunk_score = chunks
            .iter()
            .map(|chunk| self.score_chunk(chunk, query_tokens))
            .sum::<f64>();

        let raw_score = title_score + tag_score + source_score + chunk_score;

        if raw_score > 0.0 {
            raw_score / (query_tokens.len() as f64 + 1.0)
        } else {
            0.0
        }
    }

    fn fallback_candidates(
        &self,
        storage: &dyn KnowledgeStorage,
        normalized_query: &str,
    ) -> KnowledgeResult<Vec<(Document, f64, Vec<Chunk>)>> {
        let mut candidates = Vec::new();
        let documents = storage.list_documents()?;

        for document in documents {
            let haystack = format!(
                "{} {} {} {} {}",
                document.title(),
                document.source(),
                document.language(),
                document.content(),
                document.tags().join(" ")
            )
            .to_lowercase();

            if haystack.contains(normalized_query) {
                let chunks = storage.list_chunks(document.id())?;
                candidates.push((document, 0.1, chunks));
            }
        }

        Ok(candidates)
    }
}

impl KnowledgeRetrievalEngine for SemanticRetrievalEngine {
    fn retrieve(
        &self,
        storage: &dyn KnowledgeStorage,
        query: &str,
        limit: usize,
    ) -> KnowledgeResult<RetrievalResult> {
        let normalized_query = self.normalize_text(query);
        let query_tokens = self.token_set(&normalized_query);

        if query_tokens.is_empty() {
            return Err(KnowledgeError::ValidationFailed(
                "query must contain at least one searchable token".to_string(),
            ));
        }

        let documents = storage.list_documents()?;
        let mut scored_documents: Vec<(Document, f64, Vec<Chunk>)> = Vec::new();

        for document in documents {
            let chunks = storage.list_chunks(document.id())?;
            let score = self.score_document(&document, &chunks, &query_tokens);

            if score > 0.0 {
                scored_documents.push((document, score, chunks));
            }
        }

        if scored_documents.is_empty() {
            scored_documents = self.fallback_candidates(storage, &normalized_query)?;
        }

        scored_documents.sort_by(|(left_doc, left_score, _), (right_doc, right_score, _)| {
            right_score
                .partial_cmp(left_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left_doc.id().cmp(right_doc.id()))
        });

        let candidate_count = scored_documents.len();
        let top_documents = scored_documents
            .iter()
            .take(limit)
            .map(|(document, _, _)| document.clone())
            .collect::<Vec<_>>();
        let top_chunks = scored_documents
            .iter()
            .take(limit)
            .flat_map(|(_, _, chunks)| chunks.clone())
            .collect::<Vec<_>>();

        Ok(RetrievalResult::new(
            query.to_string(),
            top_documents,
            top_chunks,
            candidate_count,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::knowledge::builder::KnowledgePipelineBuilder;
    use crate::knowledge::manager::KnowledgeManagerImpl;
    use crate::knowledge::storage::InMemoryKnowledgeStorage;
    use crate::knowledge::traits::KnowledgeManager;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn semantic_retrieval_scores_relevant_document() {
        let dir = tempdir().expect("create temp dir");
        let file_path = dir.path().join("notes.md");
        let mut file = File::create(&file_path).expect("create file");
        writeln!(file, "Raven can summarize notes and answer questions.").expect("write file");

        let pipeline = KnowledgePipelineBuilder::new()
            .with_storage(Box::new(InMemoryKnowledgeStorage::new()))
            .build();
        let manager = KnowledgeManagerImpl::new_with_default_engine(pipeline);
        let id = manager.add_document(&file_path).expect("add document");
        assert!(!id.is_empty());

        let result = manager
            .retrieve("summarize notes", 5)
            .expect("retrieve query");

        assert_eq!(result.document_count, 1);
        assert!(result.documents[0].content().contains("summarize notes"));
        assert!(result.candidate_count >= 1);
    }

    #[test]
    fn semantic_retrieval_fallback_matches_by_text_contains() {
        let dir = tempdir().expect("create temp dir");
        let file_path = dir.path().join("guide.txt");
        let mut file = File::create(&file_path).expect("create file");
        writeln!(file, "This guide describes Raven architecture and design.").expect("write file");

        let pipeline = KnowledgePipelineBuilder::new()
            .with_storage(Box::new(InMemoryKnowledgeStorage::new()))
            .build();
        let manager = KnowledgeManagerImpl::new_with_default_engine(pipeline);
        let id = manager.add_document(&file_path).expect("add document");
        assert!(!id.is_empty());

        let result = manager.retrieve("architecture", 5).expect("retrieve query");

        assert_eq!(result.document_count, 1);
        assert!(result.documents[0].content().contains("Raven architecture"));
    }
}
