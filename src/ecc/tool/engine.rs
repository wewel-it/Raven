use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::Pipeline;
use crate::ecc::tool::types::ToolCall;
use crate::ecc::traits::{
    ConfidenceScorer, Corrector, ErrorClassifier, Policy, Reporter, Validator,
};
use std::sync::Arc;

/// The main orchestrator for Tool ECC.

pub struct ToolEccEngine {
    validator: Arc<dyn Validator<ToolCall>>,
    _corrector: Arc<dyn Corrector<ToolCall>>,
    _classifier: Arc<dyn ErrorClassifier<ToolCall>>,
    _scorer: Arc<dyn ConfidenceScorer<ToolCall>>,
    _reporter: Arc<dyn Reporter<ToolCall>>,
    _policy: Arc<dyn Policy>,
    pipeline: Pipeline<ToolCall>,
}

impl ToolEccEngine {
    /// Create a new Tool ECC engine with the given components.
    pub fn new(
        validator: Arc<dyn Validator<ToolCall>>,
        corrector: Arc<dyn Corrector<ToolCall>>,
        classifier: Arc<dyn ErrorClassifier<ToolCall>>,
        scorer: Arc<dyn ConfidenceScorer<ToolCall>>,
        reporter: Arc<dyn Reporter<ToolCall>>,
        policy: Arc<dyn Policy>,
        pipeline: Pipeline<ToolCall>,
    ) -> Self {
        Self {
            validator,
            _corrector: corrector,
            _classifier: classifier,
            _scorer: scorer,
            _reporter: reporter,
            _policy: policy,
            pipeline,
        }
    }

    /// Execute the Tool ECC pipeline against an incoming tool call.
    pub fn execute(&self, tool_call: ToolCall) -> EccResult<crate::ecc::report::EccReport> {
        let mut context = crate::ecc::pipeline::PipelineContext::new(tool_call);
        self.pipeline.run(&mut context)
    }

    /// Validate a tool call without running the full pipeline.
    pub fn validate(
        &self,
        tool_call: &ToolCall,
    ) -> EccResult<crate::ecc::report::ValidationReport> {
        self.validator.validate(tool_call)
    }
}
