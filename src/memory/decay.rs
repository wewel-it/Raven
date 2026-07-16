use crate::memory::MemoryEntry;

#[derive(Debug, Clone)]
pub struct DecayPolicy {
    pub decay_rate_per_hour: f32,
}

impl DecayPolicy {
    pub fn default() -> Self { Self { decay_rate_per_hour: 0.01 } }

    pub fn apply_short(&self, short: &mut crate::memory::short_term::ShortTermMemory) {
        let mut items = short.drain_all();
        for e in items.iter_mut() {
            e.importance = (e.importance - self.decay_rate_per_hour).max(0.0);
        }
        for e in items { short.push(e); }
    }

    pub fn apply_long(&self, long: &mut crate::memory::long_term::LongTermMemory) {
        // long-term decays slower
        let mut all: Vec<MemoryEntry> = long.iter().cloned().collect();
        for e in all.iter_mut() {
            e.importance = (e.importance - (self.decay_rate_per_hour * 0.1)).max(0.0);
        }
        // rebuild long by clearing and pushing back updated
        // (LongTermMemory does not expose clear, so create new and swap)
        let mut new = crate::memory::long_term::LongTermMemory::new();
        for e in all { new.push(e); }
        *long = new;
    }
}

impl Default for DecayPolicy { fn default() -> Self { DecayPolicy::default() } }
