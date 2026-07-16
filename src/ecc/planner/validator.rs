use crate::ecc::errors::EccResult;
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::{Rule, Validator};
use crate::planner::ExecutionPlan;

/// Planner-specific validator menggunakan rule ECC.
pub struct PlannerValidator {
    pub rules: Vec<Box<dyn Rule<ExecutionPlan>>>,
}

impl PlannerValidator {
    pub fn new(rules: Vec<Box<dyn Rule<ExecutionPlan>>>) -> Self {
        Self { rules }
    }
}

impl Validator<ExecutionPlan> for PlannerValidator {
    fn validate(&self, subject: &ExecutionPlan) -> EccResult<ValidationReport> {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();

        for rule in &self.rules {
            if rule.applies_to(subject) {
                let mut rule_issues = rule.evaluate(subject)?;
                issues.append(&mut rule_issues);
            }
        }

        let duration = start.elapsed();
        Ok(ValidationReport::new(chrono::Utc::now(), duration, issues))
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.id()).collect()
    }
}
