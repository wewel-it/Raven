//! Rule: Workflow harus memiliki ID yang valid.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan workflow memiliki ID yang valid dan tidak kosong.
pub struct WorkflowIdRule;

impl Rule<Workflow> for WorkflowIdRule {
    fn id(&self) -> &'static str {
        "workflow_id_rule"
    }

    fn description(&self) -> &'static str {
        "Workflow must have a valid non-empty ID"
    }

    fn applies_to(&self, _: &Workflow) -> bool {
        true
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        if workflow.id.is_empty() {
            issues.push(EccIssue::new(
                "workflow_id_empty".to_string(),
                "Workflow ID must not be empty".to_string(),
                Some("Provide a non-empty workflow ID".to_string()),
                Some("Workflow".to_string()),
            ));
        }

        if !workflow
            .id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            issues.push(EccIssue::new(
                "workflow_id_invalid_chars".to_string(),
                "Workflow ID contains invalid characters".to_string(),
                Some("Use only alphanumeric characters, hyphens, and underscores".to_string()),
                Some(format!("ID: {}", workflow.id)),
            ));
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workflow_id() {
        let rule = WorkflowIdRule;
        let workflow = Workflow::new("valid-workflow-123".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_workflow_id() {
        let rule = WorkflowIdRule;
        let workflow = Workflow::new("".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_invalid_characters() {
        let rule = WorkflowIdRule;
        let workflow = Workflow::new("invalid@workflow#".to_string());
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
