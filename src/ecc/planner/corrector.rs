use crate::ecc::errors::EccResult;
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::Corrector;
use crate::planner::ExecutionPlan;

/// Planner-specific corrector yang melakukan normalisasi plan deterministik.
pub struct PlannerCorrector;

impl PlannerCorrector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlannerCorrector {
    fn default() -> Self {
        Self::new()
    }
}

impl Corrector<ExecutionPlan> for PlannerCorrector {
    fn correct(
        &self,
        subject: &ExecutionPlan,
        _report: &ValidationReport,
    ) -> EccResult<ExecutionPlan> {
        let mut corrected = subject.clone();
        let mut fixes = Vec::new();

        for step in corrected.steps.iter_mut() {
            let original_len = step.depends_on.len();
            step.depends_on.sort();
            step.depends_on.dedup();
            if step.depends_on.len() != original_len {
                fixes.push(format!("deduplicated dependencies for {}", step.id));
            }

            if !matches!(
                step.status,
                crate::planner::StepStatus::Pending
                    | crate::planner::StepStatus::InProgress
                    | crate::planner::StepStatus::Completed
                    | crate::planner::StepStatus::Failed
                    | crate::planner::StepStatus::Retrying
                    | crate::planner::StepStatus::Skipped
            ) {
                step.status = crate::planner::StepStatus::Pending;
                fixes.push(format!("normalized status for {}", step.id));
            }
        }

        corrected.steps.retain(|step| {
            let keep = !step.id.trim().is_empty() && !step.description.trim().is_empty();
            if !keep {
                fixes.push(format!("removed empty step {}", step.id));
            }
            keep
        });

        Ok(corrected)
    }
}
