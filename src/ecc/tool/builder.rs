use crate::ecc::pipeline::Pipeline;
use crate::ecc::tool::classifier::ToolErrorClassifier;
use crate::ecc::tool::confidence::ToolConfidenceScorer;
use crate::ecc::tool::context::ToolEccContext;
use crate::ecc::tool::corrector::ToolCorrector;
use crate::ecc::tool::engine::ToolEccEngine;
use crate::ecc::tool::pipeline::{
    ToolClassificationStage, ToolConfidenceStage, ToolCorrectionStage, ToolEccPipeline,
    ToolPolicyStage, ToolReportingStage, ToolValidationStage,
};
use crate::ecc::tool::policy::ToolPolicy;
use crate::ecc::tool::report::ToolReporter;
use crate::ecc::tool::rules::ToolRule;
use crate::ecc::tool::types::ToolCall;
use crate::ecc::tool::validator::ToolValidator;
use crate::ecc::traits::{
    ConfidenceScorer, Corrector, ErrorClassifier, Policy, Reporter, Validator,
};
use std::sync::Arc;

/// Builder for constructing a Tool ECC engine and its pipeline.
pub struct ToolEccBuilder {
    tool_context: ToolEccContext,
    validator: Option<Arc<dyn Validator<ToolCall>>>,
    rules: Vec<Box<dyn ToolRule>>,
    corrector: Option<Arc<dyn Corrector<ToolCall>>>,
    classifier: Option<Arc<dyn ErrorClassifier<ToolCall>>>,
    scorer: Option<Arc<dyn ConfidenceScorer<ToolCall>>>,
    policy: Option<Arc<dyn Policy>>,
    reporter: Option<Arc<dyn Reporter<ToolCall>>>,
    pipeline: Option<Pipeline<ToolCall>>,
}

impl ToolEccBuilder {
    /// Create a new builder with a shared tool context.
    pub fn new(tool_context: ToolEccContext) -> Self {
        Self {
            tool_context,
            validator: None,
            rules: Vec::new(),
            corrector: None,
            classifier: None,
            scorer: None,
            policy: None,
            reporter: None,
            pipeline: None,
        }
    }

    /// Register a custom validator for tool calls.
    pub fn register_validator(mut self, validator: Box<dyn Validator<ToolCall>>) -> Self {
        self.validator = Some(Arc::from(validator));
        self
    }

    /// Register an individual rule to be used by the default validator.
    pub fn register_rule(mut self, rule: Box<dyn ToolRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Register a custom corrector.
    pub fn register_corrector(mut self, corrector: Box<dyn Corrector<ToolCall>>) -> Self {
        self.corrector = Some(Arc::from(corrector));
        self
    }

    /// Register a custom classifier.
    pub fn register_classifier(mut self, classifier: Box<dyn ErrorClassifier<ToolCall>>) -> Self {
        self.classifier = Some(Arc::from(classifier));
        self
    }

    /// Register a custom confidence scorer.
    pub fn register_scorer(mut self, scorer: Box<dyn ConfidenceScorer<ToolCall>>) -> Self {
        self.scorer = Some(Arc::from(scorer));
        self
    }

    /// Register a custom policy implementation.
    pub fn register_policy(mut self, policy: Box<dyn Policy>) -> Self {
        self.policy = Some(Arc::from(policy));
        self
    }

    /// Register a custom reporter.
    pub fn register_reporter(mut self, reporter: Box<dyn Reporter<ToolCall>>) -> Self {
        self.reporter = Some(Arc::from(reporter));
        self
    }

    /// Register a custom pipeline if the default stage order should be replaced.
    pub fn register_pipeline(mut self, pipeline: Pipeline<ToolCall>) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Build the engine with registered components or default ones.
    pub fn build(self) -> ToolEccEngine {
        let validator = self
            .validator
            .unwrap_or_else(|| Arc::new(ToolValidator::new(self.tool_context.clone(), self.rules)));

        let corrector = self
            .corrector
            .unwrap_or_else(|| Arc::new(ToolCorrector::new(self.tool_context.clone())));

        let classifier = self
            .classifier
            .unwrap_or_else(|| Arc::new(ToolErrorClassifier::new()));

        let scorer = self
            .scorer
            .unwrap_or_else(|| Arc::new(ToolConfidenceScorer::new()));

        let policy = self.policy.unwrap_or_else(|| Arc::new(ToolPolicy::new()));

        let reporter = self
            .reporter
            .unwrap_or_else(|| Arc::new(ToolReporter::new()));

        let pipeline = self.pipeline.unwrap_or_else(|| {
            ToolEccPipeline::new(
                Box::new(ToolValidationStage::new(validator.clone())),
                Box::new(ToolCorrectionStage::new(corrector.clone())),
                Box::new(ToolClassificationStage::new(classifier.clone())),
                Box::new(ToolPolicyStage::new(policy.clone())),
                Box::new(ToolConfidenceStage::new(scorer.clone())),
                Box::new(ToolReportingStage::new(reporter.clone())),
            )
        });

        ToolEccEngine::new(
            validator, corrector, classifier, scorer, reporter, policy, pipeline,
        )
    }
}
