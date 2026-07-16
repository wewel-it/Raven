use crate::memory::MemoryEntry;
use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct ShortTermMemory {
    queue: VecDeque<MemoryEntry>,
}

impl ShortTermMemory {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    pub fn push(&mut self, entry: MemoryEntry) {
        self.queue.push_back(entry);
    }
    pub fn drain_all(&mut self) -> Vec<MemoryEntry> {
        self.queue.drain(..).collect()
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &MemoryEntry> {
        self.queue.iter()
    }
}
