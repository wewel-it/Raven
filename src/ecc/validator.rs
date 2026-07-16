use crate::ecc::errors::EccResult;
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::Validator;

/// Validator berbasis rule yang menghasilkan isu dari kumpulan rule.
pub struct RuleBasedValidator<T> {
    pub rules: Vec<Box<dyn crate::ecc::traits::Rule<T>>>,
}

impl<T> RuleBasedValidator<T> {
    /// Buat validator baru dengan set rules yang tersedia.
    pub fn new(rules: Vec<Box<dyn crate::ecc::traits::Rule<T>>>) -> Self {
        Self { rules }
    }
}

impl<T> Validator<T> for RuleBasedValidator<T>
where
    T: Send + Sync,
{
    fn validate(&self, subject: &T) -> EccResult<ValidationReport> {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();

        for rule in &self.rules {
            if rule.applies_to(subject) {
                let mut rule_issues = rule.evaluate(subject)?;
                issues.append(&mut rule_issues);
            }
        }

        let duration = start.elapsed();
        let report = ValidationReport::new(chrono::Utc::now(), duration, issues);
        Ok(report)
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.id()).collect()
    }
}
