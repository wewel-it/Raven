use crate::memory::working::WorkingMemory;
use crate::memory::short_term::ShortTermMemory;
use crate::memory::long_term::LongTermMemory;

#[derive(Debug, Clone)]
pub struct Consolidator {
    pub working_to_short_threshold: usize,
    pub short_to_long_threshold: usize,
}

impl Consolidator {
    pub fn default() -> Self { Self { working_to_short_threshold: 50, short_to_long_threshold: 200 } }
    pub fn run(&self, working: &mut WorkingMemory, short: &mut ShortTermMemory, long: &mut LongTermMemory) {
        // move oldest from working to short
        while working.len() > self.working_to_short_threshold {
            if let Some(e) = working.pop_oldest() { short.push(e); } else { break; }
        }

        // if short too large, promote top importance to long
        if short.len() > self.short_to_long_threshold {
            let mut all = short.drain_all();
            all.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
            let promote = all.split_off((self.short_to_long_threshold / 4).min(all.len()));
            for e in promote { long.push(e); }
            for e in all { short.push(e); }
        }
    }
}

impl Default for Consolidator { fn default() -> Self { Consolidator::default() } }
