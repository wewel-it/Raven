use crate::memory::MemoryEntry;

#[derive(Debug, Default, Clone)]
pub struct EpisodicMemory {
    episodes: Vec<MemoryEntry>,
}

impl EpisodicMemory {
    pub fn new() -> Self { Self { episodes: Vec::new() } }
    pub fn push(&mut self, e: MemoryEntry) { self.episodes.push(e); }
    pub fn recent(&self, n: usize) -> Vec<MemoryEntry> { self.episodes.iter().rev().take(n).cloned().collect() }
}
