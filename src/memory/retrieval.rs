use crate::memory::{MemoryEntry, MemoryKind};
use crate::memory::index::MemoryIndex;
use crate::memory::cache::MemoryCache;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Retriever {
    index: Arc<RwLock<MemoryIndex>>,
    cache: Arc<RwLock<MemoryCache>>,
}

impl Retriever {
    pub fn new(index: Arc<RwLock<MemoryIndex>>, cache: Arc<RwLock<MemoryCache>>) -> Self {
        Self { index, cache }
    }

    pub fn query(&self, q: &str, kind: Option<MemoryKind>, limit: usize) -> Vec<MemoryEntry> {
        let tokens: Vec<String> = q.split_whitespace().map(|s| s.to_lowercase()).collect();
        let mut ids = Vec::new();
        let idx = self.index.read().unwrap();
        for t in tokens {
            for id in idx.lookup(&t) {
                if !ids.contains(&id) { ids.push(id); }
            }
        }

        // fetch from cache/storage
        let mut results: Vec<MemoryEntry> = Vec::new();
        let cache = self.cache.read().unwrap();
        for id in ids {
            if let Some(e) = cache.get(&id) {
                if kind.is_none() || kind.unwrap() == e.kind { results.push(e); }
            }
        }

        // score and sort
        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        results.truncate(limit);
        results
    }
}
