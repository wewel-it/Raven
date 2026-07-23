//! Batch embedding processor.
//!
//! This module handles efficient batch embedding of texts.

use std::sync::Arc;

/// Batch configuration.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Batch size for processing.
    pub batch_size: usize,
    /// Maximum total embeddings in queue before starting processing.
    pub max_queue_size: usize,
}

impl BatchConfig {
    /// Create a new batch configuration.
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            max_queue_size: batch_size * 10,
        }
    }

    /// Set maximum queue size.
    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            max_queue_size: 320,
        }
    }
}

/// Batch processor for embeddings.
pub struct BatchProcessor {
    config: BatchConfig,
}

impl BatchProcessor {
    /// Create a new batch processor.
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// Split texts into batches.
    pub fn split_into_batches<'a>(&self, texts: &[&'a str]) -> Vec<Vec<&'a str>> {
        texts
            .chunks(self.config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Get recommended batch size for texts.
    pub fn recommended_batch_size(&self, text_count: usize) -> usize {
        if text_count <= self.config.batch_size {
            text_count
        } else {
            self.config.batch_size
        }
    }

    /// Get batch configuration.
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new(BatchConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config() {
        let config = BatchConfig::new(32);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.max_queue_size, 320);
    }

    #[test]
    fn test_split_into_batches() {
        let processor = BatchProcessor::new(BatchConfig::new(10));
        let texts: Vec<&str> = (0..25).map(|_i| "text").collect();
        let batches = processor.split_into_batches(&texts);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 10);
        assert_eq!(batches[2].len(), 5);
    }

    #[test]
    fn test_split_exact_batches() {
        let processor = BatchProcessor::new(BatchConfig::new(10));
        let texts: Vec<&str> = (0..30).map(|_i| "text").collect();
        let batches = processor.split_into_batches(&texts);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 10);
        assert_eq!(batches[1].len(), 10);
        assert_eq!(batches[2].len(), 10);
    }
}
