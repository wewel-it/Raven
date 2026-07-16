use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::PipelineContext;
use crate::ecc::report::ConfidenceScore;

use crate::planner::ExecutionPlan;

/// Skorer deterministic untuk planner.
pub struct PlannerConfidenceScorer;

impl PlannerConfidenceScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlannerConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::ecc::traits::ConfidenceScorer<ExecutionPlan> for PlannerConfidenceScorer {
    fn score(&self, context: &PipelineContext<ExecutionPlan>) -> EccResult<ConfidenceScore> {
        let issues = context
            .validation_report
            .as_ref()
            .map(|report| report.issues.len())
            .unwrap_or(0);

        let value = if issues == 0 {
            100.0
        } else if issues <= 2 {
            85.0
        } else if issues <= 4 {
            70.0
        } else {
            45.0
        };

        Ok(ConfidenceScore::new(
            value,
            Some("deterministic planner confidence".into()),
        ))
    }
}
