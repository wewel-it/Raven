//! Rule: Workflow tidak boleh memiliki dependency cycle.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;
use std::collections::HashSet;

/// Memastikan tidak ada cycle dalam dependency graph.
pub struct DependencyCycleRule;

impl DependencyCycleRule {
    /// Deteksi cycle menggunakan DFS.
    fn has_cycle(workflow: &Workflow) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for step in &workflow.steps {
            if !visited.contains(&step.id) {
                if Self::dfs(&step.id, workflow, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }

    fn dfs(
        node: &str,
        workflow: &Workflow,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(step) = workflow.get_step(node) {
            for dep in &step.dependencies {
                if !visited.contains(dep) {
                    if Self::dfs(dep, workflow, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Temukan cycle yang ada.
    fn find_cycles(workflow: &Workflow) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let step_ids: Vec<String> = workflow.steps.iter().map(|s| s.id.clone()).collect();

        for start_id in step_ids {
            let mut path = Vec::new();
            let mut visited = HashSet::new();
            if Self::find_cycle_path(&start_id, workflow, &mut path, &mut visited) {
                cycles.push(path);
            }
        }

        cycles
    }

    fn find_cycle_path(
        node: &str,
        workflow: &Workflow,
        path: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visited.contains(node) {
            if let Some(pos) = path.iter().position(|n| n == node) {
                let cycle = path[pos..].to_vec();
                path.clear();
                path.extend(cycle);
                return true;
            }
            return false;
        }

        visited.insert(node.to_string());
        path.push(node.to_string());

        if let Some(step) = workflow.get_step(node) {
            for dep in &step.dependencies {
                if Self::find_cycle_path(dep, workflow, path, visited) {
                    return true;
                }
            }
        }

        path.pop();
        false
    }
}

impl Rule<Workflow> for DependencyCycleRule {
    fn id(&self) -> &'static str {
        "dependency_cycle_rule"
    }

    fn description(&self) -> &'static str {
        "Workflow must not contain dependency cycles"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        if Self::has_cycle(workflow) {
            let cycles = Self::find_cycles(workflow);
            for cycle in cycles {
                let cycle_str = cycle.join(" -> ");
                issues.push(EccIssue::new(
                    "dependency_cycle".to_string(),
                    format!("Dependency cycle detected: {}", cycle_str),
                    Some("Remove or modify dependencies to break the cycle".to_string()),
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
    fn test_no_cycle() {
        let rule = DependencyCycleRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()))
            .add_step(WorkflowStep::new("step3".to_string()).add_dependency("step2".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_simple_cycle() {
        let rule = DependencyCycleRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()).add_dependency("step2".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
