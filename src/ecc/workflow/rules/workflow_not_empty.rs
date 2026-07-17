//! Rule: Workflow tidak boleh kosong.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan workflow tidak kosong (setidaknya ada satu step).
pub struct WorkflowNotEmptyRule;

impl Rule<Workflow> for WorkflowNotEmptyRule {
    fn id(&self) -> &'static str {
        "workflow_not_empty_rule"
    }

    fn description(&self) -> &'static str {
        "Workflow must contain at least one step"
    }

    fn applies_to(&self, _: &Workflow) -> bool {
        true
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        if workflow.is_empty() {
            issues.push(EccIssue::new(
                "workflow_empty".to_string(),
                "Workflow is empty".to_string(),
                Some("Add at least one step to the workflow".to_string()),
                Some(format!("Workflow: {}", workflow.id)),
            ));
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecc::workflow::types::WorkflowStep;

    #[test]
    fn test_non_empty_workflow() {
        let rule = WorkflowNotEmptyRule;
        let workflow =
            Workflow::new("test".to_string()).add_step(WorkflowStep::new("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_workflow() {
        let rule = WorkflowNotEmptyRule;
        let workflow = Workflow::new("test".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
