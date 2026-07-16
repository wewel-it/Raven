use crate::memory::MemoryEntry;
use std::collections::HashMap;
use std::sync::Mutex;

/// StorageLayer trait for persistence; simple in-memory implementation provided.
pub trait StorageLayer {
    fn save(&mut self, entry: &MemoryEntry) -> Result<(), String>;
    fn load_all(&self) -> Result<Vec<MemoryEntry>, String>;
}

pub struct InMemoryStorage {
    map: Mutex<HashMap<String, MemoryEntry>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
}

impl StorageLayer for InMemoryStorage {
    fn save(&mut self, entry: &MemoryEntry) -> Result<(), String> {
        let mut m = self.map.lock().map_err(|e| format!("lock error: {}", e))?;
        m.insert(entry.id.clone(), entry.clone());
        Ok(())
    }

    fn load_all(&self) -> Result<Vec<MemoryEntry>, String> {
        let m = self.map.lock().map_err(|e| format!("lock error: {}", e))?;
        Ok(m.values().cloned().collect())
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        InMemoryStorage::new()
    }
}
