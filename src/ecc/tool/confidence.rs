use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::PipelineContext;
use crate::ecc::report::ConfidenceScore;
use crate::ecc::traits::ConfidenceScorer;

/// Confidence scoring for Tool ECC results.
pub struct ToolConfidenceScorer;

impl ToolConfidenceScorer {
    /// Create a new tool confidence scorer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceScorer<crate::ecc::tool::types::ToolCall> for ToolConfidenceScorer {
    fn score(
        &self,
        context: &PipelineContext<crate::ecc::tool::types::ToolCall>,
    ) -> EccResult<ConfidenceScore> {
        let validation = context.validation_report.as_ref().ok_or_else(|| {
            crate::ecc::errors::EccError::Pipeline {
                details: "missing validation report for confidence scoring".into(),
            }
        })?;

        let mut score = 1.0_f32;
        if !validation.is_valid {
            let issue_count = validation.issues.len() as f32;
            score -= (issue_count * 0.15).min(0.8);
        }

        if !context.applied_fixes.is_empty() {
            score -= 0.05;
        }

        let rationale = Some(format!(
            "{} issue(s), {} fix(es) applied",
            validation.issues.len(),
            context.applied_fixes.len()
        ));

        Ok(ConfidenceScore::new(score.max(0.0), rationale))
    }
}
