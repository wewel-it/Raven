//! Rule: Tidak boleh ada duplicate step.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;
use std::collections::HashSet;

/// Memastikan tidak ada step yang duplikat berdasarkan ID.
pub struct DuplicateStepRule;

impl Rule<Workflow> for DuplicateStepRule {
    fn id(&self) -> &'static str {
        "duplicate_step_rule"
    }

    fn description(&self) -> &'static str {
        "Workflow must not contain duplicate steps"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();
        let mut seen_ids = HashSet::new();

        for step in &workflow.steps {
            if !seen_ids.insert(step.id.clone()) {
                issues.push(EccIssue::new(
                    "duplicate_step".to_string(),
                    format!("Duplicate step: {}", step.id),
                    Some("Remove duplicate step or rename it with unique ID".to_string()),
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
    fn test_no_duplicates() {
        let rule = DuplicateStepRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_has_duplicates() {
        let rule = DuplicateStepRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
