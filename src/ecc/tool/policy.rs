use crate::ecc::policy::{PolicyAction, PolicyDecision};
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::Policy;

/// Policy that decides the correct action for tool ECC results.
pub struct ToolPolicy;

impl ToolPolicy {
    /// Create a new tool policy.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl Policy for ToolPolicy {
    fn decide(&self, report: &ValidationReport) -> PolicyDecision {
        if report.is_valid {
            PolicyDecision::accept()
        } else {
            PolicyDecision::new(
                PolicyAction::Reject,
                Some("tool call rejected due to validation issues".into()),
            )
        }
    }
}
