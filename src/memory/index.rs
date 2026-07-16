use crate::memory::MemoryEntry;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct MemoryIndex {
    // simple inverted index: token -> set of ids
    map: HashMap<String, HashSet<String>>,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn index_entry(&mut self, e: &MemoryEntry) {
        let tokens = tokenize(&e.text);
        for t in tokens {
            self.map.entry(t).or_default().insert(e.id.clone());
        }
        for tag in &e.tags {
            self.map
                .entry(tag.to_lowercase())
                .or_default()
                .insert(e.id.clone());
        }
    }

    pub fn lookup(&self, token: &str) -> Vec<String> {
        self.map
            .get(&token.to_lowercase())
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| {
            s.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect()
}
