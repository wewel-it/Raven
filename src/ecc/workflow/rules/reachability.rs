//! Rule: Setiap step yang bukan start harus reachable.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;
use std::collections::{HashSet, VecDeque};

/// Memastikan semua step dapat dicapai dari start step.
pub struct ReachabilityRule;

impl ReachabilityRule {
    /// BFS untuk menemukan step yang dapat dicapai.
    fn find_reachable(workflow: &Workflow, start_id: &str) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_id.to_string());

        while let Some(current) = queue.pop_front() {
            if reachable.insert(current.clone()) {
                if let Some(_step) = workflow.get_step(&current) {
                    // Add steps that depend on current
                    for s in &workflow.steps {
                        if s.dependencies.contains(&current) && !reachable.contains(&s.id) {
                            queue.push_back(s.id.clone());
                        }
                    }
                }
            }
        }

        reachable
    }
}

impl Rule<Workflow> for ReachabilityRule {
    fn id(&self) -> &'static str {
        "reachability_rule"
    }

    fn description(&self) -> &'static str {
        "All steps must be reachable from start step"
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
            let reachable = Self::find_reachable(workflow, start_id);

            for step in &workflow.steps {
                if !reachable.contains(&step.id) && &step.id != start_id {
                    issues.push(EccIssue::new(
                        "unreachable_step".to_string(),
                        format!("Step {} is not reachable from start", step.id),
                        Some("Add dependency path from start step to this step".to_string()),
                        Some(format!("Workflow: {}", workflow.id)),
                    ));
                }
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
    fn test_all_reachable() {
        let rule = ReachabilityRule;
        let workflow = Workflow::new("test".to_string())
            .with_start_step("step1".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_unreachable_step() {
        let rule = ReachabilityRule;
        let workflow = Workflow::new("test".to_string())
            .with_start_step("step1".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
