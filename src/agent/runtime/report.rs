use crate::agent::runtime::metrics::RuntimeMetricsSnapshot;
use crate::agent::runtime::state::LifecycleState;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RuntimeReport {
    pub status: LifecycleState,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub metrics: RuntimeMetricsSnapshot,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl RuntimeReport {
    pub fn new(status: LifecycleState) -> Self {
        Self {
            status,
            started_at: Utc::now(),
            finished_at: None,
            metrics: RuntimeMetricsSnapshot::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn complete(&mut self) {
        self.finished_at = Some(Utc::now());
    }
}
