//! Rule: Start step harus valid jika ditentukan.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan start step valid jika ditentukan.
pub struct StartStateRule;

impl Rule<Workflow> for StartStateRule {
    fn id(&self) -> &'static str {
        "start_state_rule"
    }

    fn description(&self) -> &'static str {
        "Start step, if defined, must exist in workflow"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty() && workflow.start_step_id.is_some()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        if let Some(start_id) = &workflow.start_step_id {
            if !workflow.step_ids().contains(start_id) {
                issues.push(EccIssue::new(
                    "invalid_start_step".to_string(),
                    format!("Start step {} does not exist", start_id),
                    Some("Ensure start step exists in the workflow".to_string()),
                    Some(format!("Workflow: {}", workflow.id)),
                ));
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecc::workflow::types::WorkflowStep;

    #[test]
    fn test_valid_start_step() {
        let rule = StartStateRule;
        let workflow = Workflow::new("test".to_string())
            .with_start_step("step1".to_string())
            .add_step(WorkflowStep::new("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_invalid_start_step() {
        let rule = StartStateRule;
        let workflow = Workflow::new("test".to_string())
            .with_start_step("missing".to_string())
            .add_step(WorkflowStep::new("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
