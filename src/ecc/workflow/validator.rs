//! Workflow validator implementation.

use crate::ecc::report::ValidationReport;
use crate::ecc::traits::{Rule, Validator};
use crate::ecc::workflow::rules;
use crate::ecc::workflow::types::Workflow;
use chrono::Utc;
use std::time::Instant;

/// Workflow-specific validator yang menggunakan rule ECC.
pub struct WorkflowValidator {
    pub rules: Vec<Box<dyn Rule<Workflow>>>,
}

impl Clone for WorkflowValidator {
    fn clone(&self) -> Self {
        Self {
            rules: rules::get_all_rules(),
        }
    }
}

impl WorkflowValidator {
    /// Buat validator dengan rules default.
    pub fn with_default_rules() -> Self {
        Self {
            rules: rules::get_all_rules(),
        }
    }

    /// Buat validator dengan rules custom.
    pub fn new(rules: Vec<Box<dyn Rule<Workflow>>>) -> Self {
        Self { rules }
    }

    /// Register tambahan rule.
    pub fn with_rule(mut self, rule: Box<dyn Rule<Workflow>>) -> Self {
        self.rules.push(rule);
        self
    }
}

impl Validator<Workflow> for WorkflowValidator {
    fn validate(&self, workflow: &Workflow) -> crate::ecc::errors::EccResult<ValidationReport> {
        let start = Instant::now();
        let mut issues = Vec::new();

        for rule in &self.rules {
            if rule.applies_to(workflow) {
                let mut rule_issues = rule.evaluate(workflow)?;
                issues.append(&mut rule_issues);
            }
        }

        let duration = start.elapsed();
        Ok(ValidationReport::new(Utc::now(), duration, issues))
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.id()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecc::workflow::types::WorkflowStep;

    #[test]
    fn test_valid_workflow() {
        let validator = WorkflowValidator::with_default_rules();
        let workflow = Workflow::new("test".to_string())
            .with_start_step("step1".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()))
            .add_end_step("step2".to_string());

        let report = validator.validate(&workflow).unwrap();
        assert_eq!(report.issues.len(), 0);
    }

    #[test]
    fn test_invalid_workflow() {
        let validator = WorkflowValidator::with_default_rules();
        let workflow = Workflow::new("test".to_string());

        let report = validator.validate(&workflow).unwrap();
        assert!(!report.issues.is_empty());
    }
}
