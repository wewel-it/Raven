//! Rule: End steps harus valid dan dapat dicapai.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan end steps valid dan dapat dicapai.
pub struct TerminalStateRule;

impl Rule<Workflow> for TerminalStateRule {
    fn id(&self) -> &'static str {
        "terminal_state_rule"
    }

    fn description(&self) -> &'static str {
        "End steps must be valid and reachable"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty() && !workflow.end_step_ids.is_empty()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();
        let valid_step_ids = workflow.step_ids();

        for end_id in &workflow.end_step_ids {
            if !valid_step_ids.contains(end_id) {
                issues.push(EccIssue::new(
                    "invalid_end_step".to_string(),
                    format!("End step {} does not exist", end_id),
                    Some("Ensure end step exists in the workflow".to_string()),
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
    fn test_valid_end_step() {
        let rule = TerminalStateRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_end_step("step1".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_invalid_end_step() {
        let rule = TerminalStateRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_end_step("missing".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
