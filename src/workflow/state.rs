use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the lifecycle state of a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowState {
    Pending,
    Running,
    Waiting,
    Retrying,
    Completed,
    Cancelled,
    Failed,
}

/// Status summary for a workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub state: WorkflowState,
    pub current_step: Option<String>,
    pub current_index: Option<usize>,
    pub completed_steps: Vec<String>,
    pub failed_step: Option<String>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkflowStatus {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            state: WorkflowState::Pending,
            current_step: None,
            current_index: None,
            completed_steps: Vec::new(),
            failed_step: None,
            last_error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_state(&mut self, state: WorkflowState) {
        self.state = state;
        self.updated_at = Utc::now();
    }

    pub fn mark_current_step(&mut self, step_id: Option<String>, index: Option<usize>) {
        self.current_step = step_id;
        self.current_index = index;
        self.updated_at = Utc::now();
    }

    pub fn step_completed(&mut self, step_id: String) {
        self.completed_steps.push(step_id);
        self.updated_at = Utc::now();
    }

    pub fn set_failed(&mut self, step_id: String, error: String) {
        self.state = WorkflowState::Failed;
        self.failed_step = Some(step_id);
        self.last_error = Some(error);
        self.updated_at = Utc::now();
    }
}
