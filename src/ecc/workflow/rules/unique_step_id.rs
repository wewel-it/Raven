//! Rule: Setiap step harus memiliki ID unik.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;
use std::collections::HashSet;

/// Memastikan semua step ID dalam workflow bersifat unik.
pub struct UniqueStepIdRule;

impl Rule<Workflow> for UniqueStepIdRule {
    fn id(&self) -> &'static str {
        "unique_step_id_rule"
    }

    fn description(&self) -> &'static str {
        "All step IDs must be unique within the workflow"
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
            if seen_ids.contains(&step.id) {
                issues.push(EccIssue::new(
                    "duplicate_step_id".to_string(),
                    format!("Duplicate step ID: {}", step.id),
                    Some("Ensure each step has a unique ID".to_string()),
                    Some(format!("Workflow: {}", workflow.id)),
                ));
            } else {
                seen_ids.insert(step.id.clone());
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
    fn test_unique_ids() {
        let rule = UniqueStepIdRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_duplicate_ids() {
        let rule = UniqueStepIdRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
