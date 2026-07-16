use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ImportanceScorer {}

impl ImportanceScorer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn score(&self, text: &str, tags: &[String], created: DateTime<Utc>) -> f32 {
        let len_score = (text.len() as f32).min(2000.0) / 2000.0;
        let tag_score = (tags.len() as f32).min(10.0) / 10.0;
        let age_seconds = (Utc::now() - created).num_seconds() as f32;
        let recency = 1.0 / (1.0 + age_seconds / 3600.0);
        (len_score * 0.5) + (tag_score * 0.3) + (recency * 0.2)
    }
}

impl Default for ImportanceScorer {
    fn default() -> Self {
        ImportanceScorer::new()
    }
}
