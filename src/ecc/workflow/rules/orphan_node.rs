//! Rule: Tidak boleh ada orphan node (step yang tidak punya path ke end).

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;
use std::collections::HashSet;

/// Memastikan semua step memiliki path ke end state.
pub struct OrphanNodeRule;

impl OrphanNodeRule {
    /// Cari step yang dapat mencapai end steps.
    fn find_steps_reaching_end(workflow: &Workflow) -> HashSet<String> {
        let mut reaching = HashSet::new();

        for end_id in &workflow.end_step_ids {
            reaching.insert(end_id.clone());
        }

        let mut changed = true;
        while changed {
            changed = false;
            for step in &workflow.steps {
                if !reaching.contains(&step.id) {
                    // Check if any dependent of this step reaches end
                    for other in &workflow.steps {
                        if other.dependencies.contains(&step.id) && reaching.contains(&other.id) {
                            reaching.insert(step.id.clone());
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }

        reaching
    }
}

impl Rule<Workflow> for OrphanNodeRule {
    fn id(&self) -> &'static str {
        "orphan_node_rule"
    }

    fn description(&self) -> &'static str {
        "All steps must have a path to at least one end step"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty() && !workflow.end_step_ids.is_empty()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();
        let reaching = Self::find_steps_reaching_end(workflow);

        for step in &workflow.steps {
            if !reaching.contains(&step.id) {
                issues.push(EccIssue::new(
                    "orphan_step".to_string(),
                    format!("Step {} has no path to any end step", step.id),
                    Some("Add dependencies to connect this step to an end step".to_string()),
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
    fn test_no_orphans() {
        let rule = OrphanNodeRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()))
            .add_end_step("step2".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_orphan_step() {
        let rule = OrphanNodeRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()))
            .add_end_step("step1".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
