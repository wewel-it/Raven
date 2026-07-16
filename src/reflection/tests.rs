use crate::reflection::ReflectionService;
use crate::workflow::state::{WorkflowState, WorkflowStatus};

#[test]
fn reflection_service_evaluates_workflow_status() {
    let engine = ReflectionService::new();
    let mut status = WorkflowStatus::new();
    status.update_state(WorkflowState::Completed);
    status.step_completed("step_0001".to_string());

    let report = engine.evaluate("test-plan", &status);
    assert_eq!(report.workflow_state, WorkflowState::Completed);
    assert!(report.summary.contains("completed successfully"));
    assert!(report.items_to_store.iter().any(|item| item.contains("completed steps count")));
}
