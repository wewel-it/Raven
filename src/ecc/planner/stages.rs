use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::{PipelineContext, PipelineStage};
use crate::ecc::traits::{Corrector, ErrorClassifier, Reporter, Validator};
use crate::planner::ExecutionPlan;

/// Pipeline stage validasi ECC planner.
pub struct PlannerValidationStage {
    pub validator: Box<dyn Validator<ExecutionPlan>>,
}

impl PlannerValidationStage {
    pub fn new(validator: Box<dyn Validator<ExecutionPlan>>) -> Self {
        Self { validator }
    }
}

impl PipelineStage<ExecutionPlan> for PlannerValidationStage {
    fn name(&self) -> &'static str {
        "planner_validate"
    }

    fn execute(&self, context: &mut PipelineContext<ExecutionPlan>) -> EccResult<()> {
        let validation = self.validator.validate(&context.subject)?;
        context.executed_rules = self
            .validator
            .rule_ids()
            .into_iter()
            .map(|id| id.to_string())
            .collect();
        context.validation_report = Some(validation);
        Ok(())
    }
}

/// Pipeline stage klasifikasi isu ECC planner.
pub struct PlannerClassificationStage {
    pub classifier: Box<dyn ErrorClassifier<ExecutionPlan>>,
}

impl PlannerClassificationStage {
    pub fn new(classifier: Box<dyn ErrorClassifier<ExecutionPlan>>) -> Self {
        Self { classifier }
    }
}

impl PipelineStage<ExecutionPlan> for PlannerClassificationStage {
    fn name(&self) -> &'static str {
        "planner_classify"
    }

    fn execute(&self, context: &mut PipelineContext<ExecutionPlan>) -> EccResult<()> {
        let validation =
            context
                .validation_report
                .as_ref()
                .ok_or(crate::ecc::errors::EccError::Pipeline {
                    details: "missing validation report before classification".into(),
                })?;

        let mut classifications = Vec::new();
        for issue in &validation.issues {
            classifications.push(self.classifier.classify(issue, context)?);
        }
        context.error_classification = classifications;
        Ok(())
    }
}

/// Pipeline stage koreksi planner.
pub struct PlannerCorrectionStage {
    pub corrector: Box<dyn Corrector<ExecutionPlan>>,
}

impl PlannerCorrectionStage {
    pub fn new(corrector: Box<dyn Corrector<ExecutionPlan>>) -> Self {
        Self { corrector }
    }
}

impl PipelineStage<ExecutionPlan> for PlannerCorrectionStage {
    fn name(&self) -> &'static str {
        "planner_correct"
    }

    fn execute(&self, context: &mut PipelineContext<ExecutionPlan>) -> EccResult<()> {
        let report =
            context
                .validation_report
                .as_ref()
                .ok_or(crate::ecc::errors::EccError::Pipeline {
                    details: "missing validation report before correction".into(),
                })?;

        let corrected = self.corrector.correct(&context.subject, report)?;
        if corrected != context.subject {
            context
                .applied_fixes
                .push("normalized execution plan".into());
        }
        context.corrected_subject = Some(corrected);
        Ok(())
    }
}

/// Pipeline stage verifikasi ECC planner.
pub struct PlannerVerificationStage {
    pub validator: Box<dyn Validator<ExecutionPlan>>,
}

impl PlannerVerificationStage {
    pub fn new(validator: Box<dyn Validator<ExecutionPlan>>) -> Self {
        Self { validator }
    }
}

impl PipelineStage<ExecutionPlan> for PlannerVerificationStage {
    fn name(&self) -> &'static str {
        "planner_verify"
    }

    fn execute(&self, context: &mut PipelineContext<ExecutionPlan>) -> EccResult<()> {
        if let Some(corrected) = &context.corrected_subject {
            let report = self.validator.validate(corrected)?;
            context.validation_report = Some(report);
        }
        Ok(())
    }
}

/// Pipeline stage scoring ECC planner.
pub struct PlannerScoringStage {
    pub scorer: Box<dyn crate::ecc::traits::ConfidenceScorer<ExecutionPlan>>,
    pub policy: Box<dyn crate::ecc::policy::Policy>,
}

impl PlannerScoringStage {
    pub fn new(
        scorer: Box<dyn crate::ecc::traits::ConfidenceScorer<ExecutionPlan>>,
        policy: Box<dyn crate::ecc::policy::Policy>,
    ) -> Self {
        Self { scorer, policy }
    }
}

impl PipelineStage<ExecutionPlan> for PlannerScoringStage {
    fn name(&self) -> &'static str {
        "planner_score"
    }

    fn execute(&self, context: &mut PipelineContext<ExecutionPlan>) -> EccResult<()> {
        let score = self.scorer.score(context)?;
        context.confidence_score = Some(score.clone());
        if let Some(report) = &context.validation_report {
            context.applied_action = Some(self.policy.decide(report));
        }
        Ok(())
    }
}

/// Pipeline stage pelaporan ECC planner.
pub struct PlannerReportingStage {
    pub reporter: Box<dyn Reporter<ExecutionPlan>>,
}

impl PlannerReportingStage {
    pub fn new(reporter: Box<dyn Reporter<ExecutionPlan>>) -> Self {
        Self { reporter }
    }
}

impl PipelineStage<ExecutionPlan> for PlannerReportingStage {
    fn name(&self) -> &'static str {
        "planner_report"
    }

    fn execute(&self, context: &mut PipelineContext<ExecutionPlan>) -> EccResult<()> {
        let report = self.reporter.generate(context)?;
        context.report = Some(report);
        Ok(())
    }
}
