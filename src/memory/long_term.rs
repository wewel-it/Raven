use crate::memory::MemoryEntry;
use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct LongTermMemory {
    store: VecDeque<MemoryEntry>,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            store: VecDeque::new(),
        }
    }
    pub fn push(&mut self, e: MemoryEntry) {
        self.store.push_back(e);
    }
    pub fn iter(&self) -> impl Iterator<Item = &MemoryEntry> {
        self.store.iter()
    }
    pub fn len(&self) -> usize {
        self.store.len()
    }
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}
