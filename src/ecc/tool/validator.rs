use crate::ecc::errors::EccResult;
use crate::ecc::report::ValidationReport;
use crate::ecc::tool::context::ToolEccContext;
use crate::ecc::tool::rules::ToolRule;
use crate::ecc::tool::types::ToolCall;
use crate::ecc::traits::Validator;
use chrono::Utc;
use std::time::Instant;

/// Rule-based validator for tool calls.
pub struct ToolValidator {
    context: ToolEccContext,
    rules: Vec<Box<dyn ToolRule>>,
}

impl ToolValidator {
    /// Create a new validator with the provided tool context and rule set.
    pub fn new(context: ToolEccContext, rules: Vec<Box<dyn ToolRule>>) -> Self {
        Self { context, rules }
    }
}

impl Validator<ToolCall> for ToolValidator {
    fn validate(&self, subject: &ToolCall) -> EccResult<ValidationReport> {
        let start = Instant::now();
        let mut issues = Vec::new();

        for rule in &self.rules {
            if rule.applies_to(subject, &self.context) {
                let mut rule_issues = rule.evaluate(subject, &self.context)?;
                issues.append(&mut rule_issues);
            }
        }

        let report = ValidationReport::new(Utc::now(), start.elapsed(), issues);
        Ok(report)
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.id()).collect()
    }
}
