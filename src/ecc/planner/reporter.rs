use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::PipelineContext;
use crate::ecc::report::EccReport;
use crate::ecc::traits::Reporter;
use crate::planner::ExecutionPlan;

/// Reporter ECC untuk planner.
pub struct PlannerReporter;

impl PlannerReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlannerReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter<ExecutionPlan> for PlannerReporter {
    fn generate(&self, context: &PipelineContext<ExecutionPlan>) -> EccResult<EccReport> {
        let validation =
            context
                .validation_report
                .clone()
                .ok_or(crate::ecc::errors::EccError::Reporting {
                    details: "missing validation report".into(),
                })?;

        let action =
            context
                .applied_action
                .clone()
                .ok_or(crate::ecc::errors::EccError::Reporting {
                    details: "missing policy decision".into(),
                })?;

        Ok(EccReport::new(
            validation,
            context.error_classification.clone(),
            context
                .confidence_score
                .clone()
                .ok_or(crate::ecc::errors::EccError::Reporting {
                    details: "missing confidence score".into(),
                })?,
            action,
            context.executed_rules.clone(),
            context.applied_fixes.clone(),
            std::time::Instant::now().elapsed(),
            chrono::Utc::now(),
        ))
    }
}
