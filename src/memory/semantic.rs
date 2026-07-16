use crate::memory::MemoryEntry;

#[derive(Debug, Default, Clone)]
pub struct SemanticMemory {
    // placeholder for semantic graphs, keys -> ids
    items: Vec<MemoryEntry>,
}

impl SemanticMemory {
    pub fn new() -> Self { Self { items: Vec::new() } }
    pub fn push(&mut self, e: MemoryEntry) { self.items.push(e); }
    pub fn search(&self, substr: &str, limit: usize) -> Vec<MemoryEntry> {
        let low = substr.to_lowercase();
        self.items.iter().filter(|e| e.text.to_lowercase().contains(&low)).take(limit).cloned().collect()
    }
}
