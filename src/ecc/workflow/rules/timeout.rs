//! Rule: Timeout configuration harus valid.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan timeout configuration valid.
pub struct TimeoutRule;

impl Rule<Workflow> for TimeoutRule {
    fn id(&self) -> &'static str {
        "timeout_rule"
    }

    fn description(&self) -> &'static str {
        "Timeout configuration must be valid"
    }

    fn applies_to(&self, _: &Workflow) -> bool {
        true
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        // Check step timeouts
        for step in &workflow.steps {
            if let Some(timeout) = step.timeout_ms {
                if timeout == 0 {
                    issues.push(EccIssue::new(
                        "zero_timeout".to_string(),
                        format!("Step {} has zero timeout", step.id),
                        Some("Set a positive timeout value in milliseconds".to_string()),
                        Some(format!("Workflow: {}", workflow.id)),
                    ));
                }
            }
        }

        // Check total timeout
        if let Some(total_timeout) = workflow.total_timeout_ms {
            if total_timeout == 0 {
                issues.push(EccIssue::new(
                    "zero_total_timeout".to_string(),
                    "Workflow has zero total timeout".to_string(),
                    Some("Set a positive total timeout value in milliseconds".to_string()),
                    Some(format!("Workflow: {}", workflow.id)),
                ));
            }

            // Check if sum of step timeouts exceeds total timeout
            let step_timeout_sum: u64 = workflow.steps.iter().filter_map(|s| s.timeout_ms).sum();

            if step_timeout_sum > total_timeout {
                issues.push(EccIssue::new(
                    "timeout_mismatch".to_string(),
                    "Sum of step timeouts exceeds workflow total timeout".to_string(),
                    Some("Adjust step or total timeout values".to_string()),
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
    fn test_valid_timeout() {
        let rule = TimeoutRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()).with_timeout(1000));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_zero_timeout() {
        let rule = TimeoutRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()).with_timeout(0));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
