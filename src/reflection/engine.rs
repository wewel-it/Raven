use crate::memory::{MemoryKind, MemoryStorage};
use crate::workflow::state::{WorkflowState, WorkflowStatus};
use serde_json::json;

/// Reflection report captures evaluation results after workflow completion.
pub struct ReflectionReport {
    pub workflow_state: WorkflowState,
    pub completed_steps: Vec<String>,
    pub failed_step: Option<String>,
    pub last_error: Option<String>,
    pub summary: String,
    pub items_to_store: Vec<String>,
    pub items_to_discard: Vec<String>,
}

impl ReflectionReport {
    pub fn summarize(&self) -> String {
        json!({
            "state": format!("{:?}", self.workflow_state),
            "completed_steps": self.completed_steps,
            "failed_step": self.failed_step,
            "last_error": self.last_error,
            "summary": self.summary,
            "items_to_store": self.items_to_store,
            "items_to_discard": self.items_to_discard,
        })
        .to_string()
    }
}

/// Reflection service performs deterministic evaluation without using an LLM.
pub trait ReflectionEvaluator: Send + Sync {
    fn evaluate(&self, plan_name: &str, status: &WorkflowStatus) -> ReflectionReport;
    fn commit(&self, manager: &dyn MemoryStorage, report: ReflectionReport) -> Result<String, String>;
}

pub struct ReflectionService;

impl ReflectionService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, plan_name: &str, status: &WorkflowStatus) -> ReflectionReport {
        let mut summary_parts = Vec::new();
        let mut store = Vec::new();
        let mut discard = Vec::new();

        match status.state {
            WorkflowState::Completed => {
                summary_parts.push(format!("Workflow '{}' completed successfully.", plan_name));
                store.push("final outcome".to_string());
                if status.completed_steps.is_empty() {
                    summary_parts.push("No steps were recorded as completed.".to_string());
                } else {
                    summary_parts.push(format!("Completed {} steps.", status.completed_steps.len()));
                }
            }
            WorkflowState::Failed => {
                summary_parts.push(format!("Workflow '{}' failed.", plan_name));
                if let Some(step) = &status.failed_step {
                    summary_parts.push(format!("Failed on step {}.", step));
                    store.push(format!("failed step: {}", step));
                }
                if let Some(err) = &status.last_error {
                    summary_parts.push(format!("Error: {}.", err));
                    store.push(format!("error: {}", err));
                }
                discard.push("partial transient state".to_string());
            }
            WorkflowState::Cancelled => {
                summary_parts.push(format!("Workflow '{}' was cancelled.", plan_name));
                discard.push("cancelled execution details".to_string());
            }
            WorkflowState::Waiting => {
                summary_parts.push(format!("Workflow '{}' is waiting.", plan_name));
                store.push("pause checkpoint".to_string());
            }
            WorkflowState::Retrying => {
                summary_parts.push(format!("Workflow '{}' is retrying.", plan_name));
                store.push("retry decision".to_string());
            }
            WorkflowState::Running => {
                summary_parts.push(format!("Workflow '{}' is still running.", plan_name));
                discard.push("in-progress transient state".to_string());
            }
            WorkflowState::Pending => {
                summary_parts.push(format!("Workflow '{}' is pending and has not run yet.", plan_name));
                discard.push("pending metadata".to_string());
            }
        }

        if !status.completed_steps.is_empty() {
            store.push(format!("completed steps count: {}", status.completed_steps.len()));
        }

        if status.failed_step.is_some() {
            discard.push("unreliable follow-up data".to_string());
        }

        let summary = summary_parts.join(" ");
        ReflectionReport {
            workflow_state: status.state.clone(),
            completed_steps: status.completed_steps.clone(),
            failed_step: status.failed_step.clone(),
            last_error: status.last_error.clone(),
            summary,
            items_to_store: store,
            items_to_discard: discard,
        }
    }

    pub fn commit(&self, manager: &dyn MemoryStorage, report: ReflectionReport) -> Result<String, String> {
        let text = report.summarize();
        let tags = ["reflection"];
        manager.add(MemoryKind::Episodic, &text, &tags)
    }
}

impl ReflectionEvaluator for ReflectionService {
    fn evaluate(&self, plan_name: &str, status: &WorkflowStatus) -> ReflectionReport {
        self.evaluate(plan_name, status)
    }

    fn commit(&self, manager: &dyn MemoryStorage, report: ReflectionReport) -> Result<String, String> {
        self.commit(manager, report)
    }
}
