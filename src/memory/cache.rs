use crate::memory::MemoryEntry;
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct MemoryCache {
    cap: usize,
    map: HashMap<String, MemoryEntry>,
    order: VecDeque<String>,
}

impl MemoryCache {
    pub fn new(cap: usize) -> Self { Self { cap, map: HashMap::new(), order: VecDeque::new() } }
    pub fn put(&mut self, id: String, entry: MemoryEntry) {
        if self.map.contains_key(&id) { return; }
        if self.order.len() >= self.cap {
            if let Some(old) = self.order.pop_front() { self.map.remove(&old); }
        }
        self.order.push_back(id.clone());
        self.map.insert(id, entry);
    }
    pub fn get(&self, id: &str) -> Option<MemoryEntry> { self.map.get(id).cloned() }
    pub fn iter_values(&self) -> Vec<MemoryEntry> { self.map.values().cloned().collect() }
}
