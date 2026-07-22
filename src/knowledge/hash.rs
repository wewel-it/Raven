use crate::knowledge::traits::HashEngine;
use blake3::Hasher;

/// BLAKE3 hashing engine implementation.
#[derive(Debug)]
pub struct Blake3HashEngine;

impl Blake3HashEngine {
    pub fn new() -> Self {
        Self
    }
}

impl HashEngine for Blake3HashEngine {
    fn hash(&self, data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }
}
