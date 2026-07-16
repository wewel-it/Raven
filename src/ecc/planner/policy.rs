use crate::ecc::policy::{Policy, PolicyAction, PolicyDecision};
use crate::ecc::report::ValidationReport;

/// Policy Planner berdasarkan hasil validasi.
pub struct PlannerPolicy;

impl PlannerPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlannerPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl Policy for PlannerPolicy {
    fn decide(&self, report: &ValidationReport) -> PolicyDecision {
        if report.is_valid {
            PolicyDecision::accept()
        } else if report.issues.len() <= 2 {
            PolicyDecision::new(
                PolicyAction::Correct,
                Some("minor planner issues, attempt correction".into()),
            )
        } else if report.issues.len() <= 4 {
            PolicyDecision::new(
                PolicyAction::Retry,
                Some("planner issues detected, retry planning".into()),
            )
        } else {
            PolicyDecision::new(PolicyAction::Reject, Some("too many planner issues".into()))
        }
    }
}
