use crate::ecc::errors::EccError;
use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::PipelineContext;
use crate::ecc::report::EccReport;
use crate::ecc::tool::types::ToolCall;
use crate::ecc::traits::Reporter;

/// Reporter for Tool ECC that builds the final ECC report.
pub struct ToolReporter;

impl ToolReporter {
    /// Create a new tool report generator.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter<ToolCall> for ToolReporter {
    fn generate(&self, context: &PipelineContext<ToolCall>) -> EccResult<EccReport> {
        let validation = context
            .validation_report
            .clone()
            .ok_or(EccError::Reporting {
                details: "missing validation report".into(),
            })?;

        let score = context
            .confidence_score
            .clone()
            .ok_or(EccError::Reporting {
                details: "missing confidence score".into(),
            })?;

        let action = context.applied_action.clone().ok_or(EccError::Reporting {
            details: "missing policy decision".into(),
        })?;

        Ok(EccReport::new(
            validation,
            context.error_classification.clone(),
            score,
            action,
            context.executed_rules.clone(),
            context.applied_fixes.clone(),
            std::time::Instant::now().elapsed(),
            chrono::Utc::now(),
        ))
    }
}
