//! Tokenization and text preprocessing.

use regex::Regex;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

/// A simple but effective tokenizer for embedding preprocessing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTokenizer;

impl SimpleTokenizer {
    /// Create a new tokenizer with default configuration.
    pub fn new() -> Self {
        Self
    }

    /// Normalize Unicode text using NFKD normalization.
    fn normalize_unicode(text: &str) -> String {
        text.nfkd().collect::<String>()
    }

    /// Convert text to lowercase.
    fn lowercase(text: &str) -> String {
        text.to_lowercase()
    }

    /// Remove non-alphanumeric characters except spaces and hyphens.
    fn clean_text(text: &str) -> String {
        text.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
            .collect()
    }

    /// Tokenize text into individual words.
    ///
    /// Returns a vector of tokens (lowercase, normalized, cleaned).
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let normalized = Self::normalize_unicode(text);
        let cleaned = Self::clean_text(&normalized);
        let lowercased = Self::lowercase(&cleaned);

        let pattern = Regex::new(r"\b\w+\b").unwrap();
        let mut tokens = Vec::new();
        for mat in pattern.find_iter(&lowercased) {
            let token = mat.as_str().to_string();
            if !token.is_empty() {
                tokens.push(token);
            }
        }
        tokens
    }

    /// Tokenize and filter out single-character tokens and common stopwords.
    pub fn tokenize_filtered(&self, text: &str) -> Vec<String> {
        let tokens = self.tokenize(text);
        tokens
            .into_iter()
            .filter(|t| t.len() > 1 && !Self::is_stopword(t))
            .collect()
    }

    /// Simple English stopword list.
    fn is_stopword(word: &str) -> bool {
        matches!(
            word,
            "a" | "an" | "and" | "are" | "as" | "at" | "be" | "but" | "by" | "for" | "from"
                | "has" | "he" | "in" | "is" | "it" | "its" | "of" | "on" | "or" | "that"
                | "the" | "to" | "was" | "will" | "with" | "i" | "me" | "you" | "we" | "they"
        )
    }

    /// Get tokens as a set for fast membership testing.
    pub fn tokenize_to_set(&self, text: &str) -> std::collections::HashSet<String> {
        self.tokenize(text).into_iter().collect()
    }

    /// Count term frequencies in the token list.
    pub fn term_frequencies(&self, tokens: &[String]) -> std::collections::HashMap<String, usize> {
        let mut freqs = std::collections::HashMap::new();
        for token in tokens {
            *freqs.entry(token.clone()).or_insert(0) += 1;
        }
        freqs
    }
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("Hello World!");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_normalize_unicode() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("Héllo Wörld");
        assert!(tokens.iter().all(|t| !t.contains('é') && !t.contains('ö')));
    }

    #[test]
    fn test_filtered_tokenization() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize_filtered("The quick brown fox jumps over the lazy dog");
        assert!(!tokens.contains(&"the".to_string()));
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
    }

    #[test]
    fn test_term_frequencies() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("hello world hello");
        let freqs = tokenizer.term_frequencies(&tokens);
        assert_eq!(freqs.get("hello"), Some(&2));
        assert_eq!(freqs.get("world"), Some(&1));
    }
}
