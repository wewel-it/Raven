use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::PipelineContext;
use crate::ecc::report::{EccIssue, ErrorClassification};
use crate::ecc::traits::ErrorClassifier;
use crate::planner::ExecutionPlan;

/// Classifier isu untuk planner.
pub struct PlannerErrorClassifier;

impl ErrorClassifier<ExecutionPlan> for PlannerErrorClassifier {
    fn classify(
        &self,
        issue: &EccIssue,
        _context: &PipelineContext<ExecutionPlan>,
    ) -> EccResult<ErrorClassification> {
        let severity = if issue.code.starts_with("structure") {
            crate::ecc::report::ErrorSeverity::High
        } else {
            crate::ecc::report::ErrorSeverity::Medium
        };

        Ok(ErrorClassification {
            issue_code: issue.code.clone(),
            category: issue.code.clone(),
            severity,
            confidence: 1.0,
        })
    }
}
