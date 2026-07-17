//! Rule: Retry configuration harus valid.

use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

/// Memastikan retry configuration valid.
pub struct RetryConfigRule;

impl Rule<Workflow> for RetryConfigRule {
    fn id(&self) -> &'static str {
        "retry_config_rule"
    }

    fn description(&self) -> &'static str {
        "Retry configuration must be valid"
    }

    fn applies_to(&self, _: &Workflow) -> bool {
        true
    }

    fn evaluate(
        &self,
        workflow: &Workflow,
    ) -> crate::ecc::errors::EccResult<Vec<crate::ecc::report::EccIssue>> {
        let mut issues = Vec::new();

        // Check step retry configuration
        for step in &workflow.steps {
            if let Some(max_retries) = step.max_retries {
                // Max retries of 0 means no retry, which is valid
                // But we can warn if it's suspiciously high
                if max_retries > 100 {
                    issues.push(EccIssue::new(
                        "high_retry_count".to_string(),
                        format!(
                            "Step {} has very high max_retries ({})",
                            step.id, max_retries
                        ),
                        Some(
                            "Consider reducing max_retries to prevent infinite retries".to_string(),
                        ),
                        Some(format!("Workflow: {}", workflow.id)),
                    ));
                }
            }
        }

        // Check recovery config
        if workflow.recovery_config.enable_retry {
            if let Some(delay) = workflow.recovery_config.retry_delay_ms {
                if delay == 0 {
                    issues.push(EccIssue::new(
                        "zero_retry_delay".to_string(),
                        "Retry delay is zero milliseconds".to_string(),
                        Some("Set a positive retry delay to avoid busy spinning".to_string()),
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
    fn test_valid_retry_config() {
        let rule = RetryConfigRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()).with_retries(3));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_high_retry_count() {
        let rule = RetryConfigRule;
        let workflow = Workflow::new("test".to_string())
            .add_step(WorkflowStep::new("step1".to_string()).with_retries(1000));
        let issues = rule.evaluate(&workflow).unwrap();
        assert!(!issues.is_empty());
    }
}
