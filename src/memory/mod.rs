//! Memory subsystem composed of multiple specialized modules.
pub mod working;
pub mod short_term;
pub mod long_term;
pub mod episodic;
pub mod semantic;
pub mod retrieval;
pub mod consolidation;
pub mod scoring;
pub mod decay;
pub mod index;
pub mod cache;
pub mod storage;

pub use working::WorkingMemory;
pub use short_term::ShortTermMemory;
pub use long_term::LongTermMemory;
pub use episodic::EpisodicMemory;
pub use semantic::SemanticMemory;
pub use retrieval::Retriever;
pub use consolidation::Consolidator;
pub use scoring::ImportanceScorer;
pub use decay::DecayPolicy;
pub use index::MemoryIndex;
pub use cache::MemoryCache;
pub use storage::{StorageLayer, InMemoryStorage};

use crate::error::{RavenError, RavenResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// MemoryStorage service interface for the agent.
pub trait MemoryStorage: Send + Sync {
    fn add(&self, kind: MemoryKind, text: &str, tags: &[&str]) -> RavenResult<String>;
    fn retrieve(&self, query: &str, kind: Option<MemoryKind>, limit: usize) -> Vec<MemoryEntry>;
    fn consolidate(&self) -> RavenResult<()>;
    fn apply_decay(&self);
    fn persist_all(&self) -> RavenResult<()>;
}

/// Memory kinds for routing entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryKind {
    Working,
    ShortTerm,
    LongTerm,
    Episodic,
    Semantic,
}

/// A single memory entry suitable for all memory modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub kind: MemoryKind,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub importance: f32,
}

/// Public MemoryService that composes all submodules. Thread-safe.
pub struct MemoryService {
    working: Arc<RwLock<WorkingMemory>>,
    short: Arc<RwLock<ShortTermMemory>>,
    long: Arc<RwLock<LongTermMemory>>,
    episodic: Arc<RwLock<EpisodicMemory>>,
    semantic: Arc<RwLock<SemanticMemory>>,
    index: Arc<RwLock<MemoryIndex>>,
    cache: Arc<RwLock<MemoryCache>>,
    storage: Arc<RwLock<Box<dyn StorageLayer + Send + Sync>>>,
    scorer: ImportanceScorer,
    consolidator: Consolidator,
    decay: DecayPolicy,
    next_id: Arc<RwLock<u64>>,
}

impl MemoryService {
    pub fn new() -> Self {
        let storage: Box<dyn StorageLayer + Send + Sync> = Box::new(InMemoryStorage::new());
        Self {
            working: Arc::new(RwLock::new(WorkingMemory::new())),
            short: Arc::new(RwLock::new(ShortTermMemory::new())),
            long: Arc::new(RwLock::new(LongTermMemory::new())),
            episodic: Arc::new(RwLock::new(EpisodicMemory::new())),
            semantic: Arc::new(RwLock::new(SemanticMemory::new())),
            index: Arc::new(RwLock::new(MemoryIndex::new())),
            cache: Arc::new(RwLock::new(MemoryCache::new(256))),
            storage: Arc::new(RwLock::new(storage)),
            scorer: ImportanceScorer::default(),
            consolidator: Consolidator::default(),
            decay: DecayPolicy::default(),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    fn allocate_id(&self) -> String {
        let mut w = self.next_id.write().unwrap();
        let id = *w;
        *w += 1;
        format!("m{:08}", id)
    }

    /// Add a new memory entry into appropriate store, update index and cache.
    pub fn add(&self, kind: MemoryKind, text: &str, tags: &[&str]) -> RavenResult<String> {
        let id = self.allocate_id();
        let now = Utc::now();
        let tags_vec: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
        let importance = self.scorer.score(text, &tags_vec, now);
        let entry = MemoryEntry { id: id.clone(), kind, text: text.to_string(), created_at: now, tags: tags_vec.clone(), importance };

        // store in chosen memory
        match kind {
            MemoryKind::Working => { self.working.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?.push(entry.clone()); }
            MemoryKind::ShortTerm => { self.short.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?.push(entry.clone()); }
            MemoryKind::LongTerm => { self.long.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?.push(entry.clone()); }
            MemoryKind::Episodic => { self.episodic.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?.push(entry.clone()); }
            MemoryKind::Semantic => { self.semantic.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?.push(entry.clone()); }
        }

        // index and cache
        self.index.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?.index_entry(&entry);
        self.cache.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?.put(entry.id.clone(), entry.clone());

        // persist
        self.storage.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?
            .save(&entry)
            .map_err(|e| RavenError::Memory(e))?;

        Ok(id)
    }

    /// Retrieve by query using retriever which consults index, cache, and scores.
    pub fn retrieve(&self, query: &str, kind: Option<MemoryKind>, limit: usize) -> Vec<MemoryEntry> {
        let retriever = Retriever::new(self.index.clone(), self.cache.clone());
        retriever.query(query, kind, limit)
    }
avenResult<()> {
        let mut working = self.working.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        let mut short = self.short.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        let mut long = self.long.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?wrap();
        let mut short = self.short.write().unwrap();
        let mut long = self.long.write().unwrap();
        self.consolidator.run(&mut *working, &mut *short, &mut *long);
        Ok(())
    }

    /// Apply decay to reduce importance over time.
    pub fn apply_decay(&self) {
        let mut short = self.short.write().unwrap();
        self.decay.apply_short(&mut *short);
        let mut long = self.long.write().unwrap();
        self.decay.apply_long(&mut *long);
    }

    /// Expose storage sync
    pub fn persist_all(&self) -> RavenResult<()> {
        // persist from cache as canonical snapshot
        let cache = self.cache.read().map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        for entry in cache.iter_values() {
            self.storage.write().map_err(|e| RavenError::LockPoisoned(e.to_string()))?
                .save(&entry)
                .map_err(|e| RavenError::Memory(e))?;
        }
        Ok(())
    }
}

impl MemoryStorage for MemoryService {
    fn add(&self, kind: MemoryKind, text: &str, tags: &[&str]) -> RavenResult<String> {
        MemoryService::add(self, kind, text, tags)
    }

    fn retrieve(&self, query: &str, kind: Option<MemoryKind>, limit: usize) -> Vec<MemoryEntry> {
        MemoryService::retrieve(self, query, kind, limit)
    }

    fn consolidate(&self) -> RavenResult<()> {
        MemoryService::consolidate(self)
    }

    fn apply_decay(&self) {
        MemoryService::apply_decay(self);
    }

    fn persist_all(&self) -> RavenResult<()> {
        MemoryService::persist_all(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_service_add_and_retrieve() {
        let m = MemoryService::new();
        let id = m.add(MemoryKind::Working, "Test entry about Raven architecture", &["raven","arch"]).unwrap();
        assert!(id.starts_with('m'));
        let res = m.retrieve("Raven", None, 10);
        assert!(!res.is_empty());
    }
}
