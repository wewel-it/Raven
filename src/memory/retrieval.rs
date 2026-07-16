use crate::memory::cache::MemoryCache;
use crate::memory::index::MemoryIndex;
use crate::memory::{MemoryEntry, MemoryKind};
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
        if let Ok(idx) = self.index.read() {
            for t in tokens {
                for id in idx.lookup(&t) {
                    if !ids.contains(&id) {
                        ids.push(id);
                    }
                }
            }
        } else {
            return Vec::new();
        }

        // fetch from cache/storage
        let mut results: Vec<MemoryEntry> = Vec::new();
        if let Ok(cache) = self.cache.read() {
            for id in ids {
                if let Some(e) = cache.get(&id) {
                    let ok = match kind {
                        None => true,
                        Some(k) => k == e.kind,
                    };
                    if ok {
                        results.push(e);
                    }
                }
            }
        } else {
            return Vec::new();
        }

        // score and sort
        results.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }
}
