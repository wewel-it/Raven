use crate::memory::MemoryEntry;
use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct WorkingMemory {
    queue: VecDeque<MemoryEntry>,
}

impl WorkingMemory {
    pub fn new() -> Self { Self { queue: VecDeque::new() } }
    pub fn push(&mut self, entry: MemoryEntry) { self.queue.push_back(entry); }
    pub fn pop_oldest(&mut self) -> Option<MemoryEntry> { self.queue.pop_front() }
    pub fn len(&self) -> usize { self.queue.len() }
    pub fn drain_to_vec(&mut self, limit: usize) -> Vec<MemoryEntry> {
        let mut out = Vec::new();
        for _ in 0..limit {
            if let Some(e) = self.queue.pop_front() { out.push(e); } else { break; }
        }
        out
    }
}
