//! Rule: Workflow harus memiliki minimal satu end step.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan workflow memiliki minimal satu end step.
pub struct EndStateRule;

impl Rule<Workflow> for EndStateRule {
    fn id(&self) -> &'static str {
        "end_state_rule"
    }

    fn description(&self) -> &'static str {
        "Workflow must have at least one end step"
    }

    fn applies_to(&self, _: &Workflow) -> bool {
        true
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        if workflow.end_step_ids.is_empty() {
            issues.push(EccIssue::new(
                "no_end_steps".to_string(),
                "Workflow has no end steps defined".to_string(),
                Some("Define at least one end step for the workflow".to_string()),
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
    fn test_with_end_step() {
        let rule = EndStateRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_end_step("step1".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_without_end_step() {
        let rule = EndStateRule;
        let workflow =
            Workflow::new("test".to_string()).add_step(WorkflowStep::new("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
