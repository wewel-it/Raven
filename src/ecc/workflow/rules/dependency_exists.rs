//! Rule: Setiap dependency harus merujuk ke step yang ada.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan semua dependency menunjuk ke step yang ada.
pub struct DependencyExistsRule;

impl Rule<Workflow> for DependencyExistsRule {
    fn id(&self) -> &'static str {
        "dependency_exists_rule"
    }

    fn description(&self) -> &'static str {
        "All step dependencies must reference existing steps"
    }

    fn applies_to(&self, workflow: &Workflow) -> bool {
        !workflow.is_empty()
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();
        let valid_step_ids = workflow.step_ids();

        for step in &workflow.steps {
            for dep in &step.dependencies {
                if !valid_step_ids.contains(dep) {
                    issues.push(EccIssue::new(
                        "missing_dependency".to_string(),
                        format!("Step {} depends on non-existent step {}", step.id, dep),
                        Some(format!(
                            "Ensure step {} exists before adding it as dependency",
                            dep
                        )),
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
    fn test_valid_dependencies() {
        let rule = DependencyExistsRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_missing_dependency() {
        let rule = DependencyExistsRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()).add_dependency("missing".to_string()));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
