//! Rule: Deteksi deadlock dalam workflow.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;
use std::collections::HashMap;

/// Memastikan tidak ada deadlock dalam dependency graph.
pub struct DeadlockRule;

impl DeadlockRule {
    /// Deteksi deadlock menggunakan circular wait detection.
    fn has_deadlock(workflow: &Workflow) -> bool {
        // Build dependency graph
        let mut in_degree = HashMap::new();
        let mut dependents = HashMap::new();

        for step in &workflow.steps {
            in_degree.entry(step.id.clone()).or_insert(0);
            for dep in &step.dependencies {
                *in_degree.entry(step.id.clone()).or_insert(0) += 1;
                dependents
                    .entry(dep.clone())
                    .or_insert_with(Vec::new)
                    .push(step.id.clone());
            }
        }

        // Topological sort using Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted_count = 0;

        while !queue.is_empty() {
            let current = queue.remove(0);
            sorted_count += 1;

            if let Some(deps) = dependents.get(&current) {
                for dependent in deps {
                    if let Some(count) = in_degree.get_mut(dependent) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push(dependent.clone());
                        }
                    }
                }
            }
        }

        // If not all steps were sorted, there's a cycle/deadlock
        sorted_count < workflow.steps.len()
    }
}

impl Rule<Workflow> for DeadlockRule {
    fn id(&self) -> &'static str {
        "deadlock_rule"
    }

    fn description(&self) -> &'static str {
        "Workflow must not have deadlock conditions"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        if Self::has_deadlock(workflow) {
            issues.push(EccIssue::new(
                "deadlock_detected".to_string(),
                "Potential deadlock detected in workflow".to_string(),
                Some("Check for circular dependencies or missing end conditions".to_string()),
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
    fn test_no_deadlock() {
        let rule = DeadlockRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_has_deadlock() {
        let rule = DeadlockRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()).add_dependency("step2".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
