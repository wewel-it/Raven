//! Rule: Workflow graph harus connected.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;
use std::collections::{HashSet, VecDeque};

/// Memastikan workflow graph terhubung dari start ke end.
pub struct DisconnectedGraphRule;

impl DisconnectedGraphRule {
    /// BFS dari start step untuk menemukan semua reachable.
    fn is_connected_from_start(workflow: &Workflow) -> bool {
        if let Some(start_id) = &workflow.start_step_id {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(start_id.clone());

            while let Some(current) = queue.pop_front() {
                if visited.insert(current.clone()) {
                    if let Some(_step) = workflow.get_step(&current) {
                        for other in &workflow.steps {
                            if other.dependencies.contains(&current) && !visited.contains(&other.id)
                            {
                                queue.push_back(other.id.clone());
                            }
                        }
                    }
                }
            }

            // Check if we can reach any end step
            for end_id in &workflow.end_step_ids {
                if visited.contains(end_id) {
                    return true;
                }
            }
        }
        false
    }
}

impl Rule<Workflow> for DisconnectedGraphRule {
    fn id(&self) -> &'static str {
        "disconnected_graph_rule"
    }

    fn description(&self) -> &'static str {
        "Workflow graph must have path from start to at least one end step"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty()
            && workflow.start_step_id.is_some()
            && !workflow.end_step_ids.is_empty()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        if !Self::is_connected_from_start(workflow) {
            issues.push(EccIssue::new(
                "disconnected_graph".to_string(),
                "No path exists from start step to any end step".to_string(),
                Some("Add dependencies to connect start step to end steps".to_string()),
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
    fn test_connected_graph() {
        let rule = DisconnectedGraphRule;
        let workflow = Workflow::new("test".to_string())
            .with_start_step("step1".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()))
            .add_end_step("step2".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_disconnected_graph() {
        let rule = DisconnectedGraphRule;
        let workflow = Workflow::new("test".to_string())
            .with_start_step("step1".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()))
            .add_end_step("step2".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
