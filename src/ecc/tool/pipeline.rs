use crate::ecc::errors::EccError;
use crate::ecc::pipeline::{Pipeline, PipelineContext, PipelineStage};
use crate::ecc::tool::types::ToolCall;
use crate::ecc::traits::{Corrector, ErrorClassifier, Policy, Reporter, Validator};

/// Stage that validates a tool call and records the execution rules.
use std::sync::Arc;

pub struct ToolValidationStage {
    validator: Arc<dyn Validator<ToolCall>>,
}

impl ToolValidationStage {
    pub fn new(validator: Arc<dyn Validator<ToolCall>>) -> Self {
        Self { validator }
    }
}

impl PipelineStage<ToolCall> for ToolValidationStage {
    fn name(&self) -> &'static str {
        "tool_validation"
    }

    fn execute(
        &self,
        context: &mut PipelineContext<ToolCall>,
    ) -> crate::ecc::errors::EccResult<()> {
        let report = self.validator.validate(&context.subject)?;
        context.executed_rules = self
            .validator
            .rule_ids()
            .into_iter()
            .map(|id| id.to_string())
            .collect();
        context.validation_report = Some(report);
        Ok(())
    }
}

/// Stage that applies deterministic corrections to a tool call.
pub struct ToolCorrectionStage {
    corrector: Arc<dyn Corrector<ToolCall>>,
}

impl ToolCorrectionStage {
    pub fn new(corrector: Arc<dyn Corrector<ToolCall>>) -> Self {
        Self { corrector }
    }
}

impl PipelineStage<ToolCall> for ToolCorrectionStage {
    fn name(&self) -> &'static str {
        "tool_correction"
    }

    fn execute(
        &self,
        context: &mut PipelineContext<ToolCall>,
    ) -> crate::ecc::errors::EccResult<()> {
        let report = context
            .validation_report
            .as_ref()
            .ok_or(EccError::Pipeline {
                details: "missing validation report before correction".into(),
            })?;

        if !report.is_valid {
            let corrected = self.corrector.correct(&context.subject, report)?;
            if corrected != context.subject {
                context
                    .applied_fixes
                    .push("deterministic correction applied".into());
            }
            context.corrected_subject = Some(corrected);
        }

        Ok(())
    }
}

/// Stage that classifies validation issues.
pub struct ToolClassificationStage {
    classifier: Arc<dyn ErrorClassifier<ToolCall>>,
}

impl ToolClassificationStage {
    pub fn new(classifier: Arc<dyn ErrorClassifier<ToolCall>>) -> Self {
        Self { classifier }
    }
}

impl PipelineStage<ToolCall> for ToolClassificationStage {
    fn name(&self) -> &'static str {
        "tool_classification"
    }

    fn execute(
        &self,
        context: &mut PipelineContext<ToolCall>,
    ) -> crate::ecc::errors::EccResult<()> {
        let report = context
            .validation_report
            .as_ref()
            .ok_or(EccError::Pipeline {
                details: "missing validation report before classification".into(),
            })?;

        let mut classifications = Vec::new();
        for issue in &report.issues {
            classifications.push(self.classifier.classify(issue, context)?);
        }
        context.error_classification = classifications;
        Ok(())
    }
}

/// Stage that applies policy decisions.
pub struct ToolPolicyStage {
    policy: Arc<dyn Policy>,
}

impl ToolPolicyStage {
    pub fn new(policy: Arc<dyn Policy>) -> Self {
        Self { policy }
    }
}

impl PipelineStage<ToolCall> for ToolPolicyStage {
    fn name(&self) -> &'static str {
        "tool_policy"
    }

    fn execute(
        &self,
        context: &mut PipelineContext<ToolCall>,
    ) -> crate::ecc::errors::EccResult<()> {
        let report = context
            .validation_report
            .as_ref()
            .ok_or(EccError::Pipeline {
                details: "missing validation report before policy".into(),
            })?;

        context.applied_action = Some(self.policy.decide(report));
        Ok(())
    }
}

/// Stage that computes a confidence score.
pub struct ToolConfidenceStage {
    scorer: Arc<dyn crate::ecc::traits::ConfidenceScorer<ToolCall>>,
}

impl ToolConfidenceStage {
    pub fn new(scorer: Arc<dyn crate::ecc::traits::ConfidenceScorer<ToolCall>>) -> Self {
        Self { scorer }
    }
}

impl PipelineStage<ToolCall> for ToolConfidenceStage {
    fn name(&self) -> &'static str {
        "tool_confidence"
    }

    fn execute(
        &self,
        context: &mut PipelineContext<ToolCall>,
    ) -> crate::ecc::errors::EccResult<()> {
        let score = self.scorer.score(context)?;
        context.confidence_score = Some(score);
        Ok(())
    }
}

/// Stage that generates a final report and stores it in the pipeline context.
pub struct ToolReportingStage {
    reporter: Arc<dyn Reporter<ToolCall>>,
}

impl ToolReportingStage {
    pub fn new(reporter: Arc<dyn Reporter<ToolCall>>) -> Self {
        Self { reporter }
    }
}

impl PipelineStage<ToolCall> for ToolReportingStage {
    fn name(&self) -> &'static str {
        "tool_reporting"
    }

    fn execute(
        &self,
        context: &mut PipelineContext<ToolCall>,
    ) -> crate::ecc::errors::EccResult<()> {
        let report = self.reporter.generate(context)?;
        context.report = Some(report);
        Ok(())
    }
}

/// Tool Ecc pipeline creates the ordered stages for tool call validation.
pub struct ToolEccPipeline;

impl ToolEccPipeline {
    /// Build the default pipeline using the given stage components.
    pub fn new(
        validator_stage: Box<dyn PipelineStage<ToolCall>>,
        corrector_stage: Box<dyn PipelineStage<ToolCall>>,
        classifier_stage: Box<dyn PipelineStage<ToolCall>>,
        policy_stage: Box<dyn PipelineStage<ToolCall>>,
        confidence_stage: Box<dyn PipelineStage<ToolCall>>,
        reporter_stage: Box<dyn PipelineStage<ToolCall>>,
    ) -> Pipeline<ToolCall> {
        Pipeline::new(vec![
            validator_stage,
            corrector_stage,
            classifier_stage,
            policy_stage,
            confidence_stage,
            reporter_stage,
        ])
    }
}
